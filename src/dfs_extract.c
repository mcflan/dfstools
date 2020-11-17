/*
 * dfs_extract.c
 *
 * Extract all files from an Acorn DFS image.
 *
 * J.Macfarlane November 2020
 */

#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <assert.h>
#include <errno.h>
#include <sys/stat.h>

#include "dfs.h"

static void extract_file(char *name, unsigned char *data, int size)
{
    int fd = open(name, O_WRONLY | O_CREAT, 0644);
    if (fd < 0) { fprintf(stderr, "%s:", name); perror(""); return; }
    int ret = write(fd, data, size);
    if (ret < 0) {
        fprintf(stderr, "%s:", name);
        perror("write");
        return;
    }
    if (ret != size) {
        fprintf(stderr, "%s: wrote wrong number of bytes\n", name);
        return;
    }
    close(fd);
}

int main(int argc, char **argv)
{
    if (argc <= 2) {
        printf("Extract all files in an Acorn DFS .ssd image\n");
        printf("usage: %s image.ssd dir\n", argv[0]);
        return 1;
    }

    char *img_file = argv[1];
    char *dir = argv[2];

    unsigned char *img = dfs_open_image(img_file);
    if (img == NULL) { return 1; }

    // Create the directory (if necessary) and change to it.
    int ret = chdir(dir);
    if (ret < 0) {
        if (errno == ENOENT) {
            int ret = mkdir(dir, 0755);
            if (ret < 0) {
                fprintf(stderr, "%s: ", dir);
                perror("mkdir");
                return 1;
            }
            ret = chdir(dir);
            if (ret < 0) {
                fprintf(stderr, "%s: ", dir);
                perror("chdir");
                return 1;
            }
        } else {
            fprintf(stderr, "%s: ", dir);
            perror("chdir");
            return 1;
        }
    }

    dfs_cat_t *cat = dfs_img_cat(img);
    // Iterate over files in catalogue
    for (int i = 0; i < cat->nfiles; i++) {
        dfs_cat_file_t *f = cat->files + i;
        char name[10];
        if (f->dir == '$') { // Don't show default dir
            snprintf(name, 10, "%s", f->name);
        } else {
            snprintf(name, 10, "%c.%s", f->dir, f->name);
        }
        extract_file(name, img+(f->sector * DFS_SECTOR_SIZE), f->size);
    }
    dfs_cat_free(cat);

    return 0;
}
