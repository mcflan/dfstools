
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/stat.h>
#include <assert.h>

#include "dfs.h"

int dfs_file_size(char *file)
{
    // Check input file size makes sense
    struct stat st;
    int ret = stat(file, &st);
    if (ret != 0) { printf("%s:", file); perror("stat"); return -1; }
    return st.st_size;
}

unsigned char *dfs_open_image(char *file)
{
    // Check we can open the file.
    int fd = open(file, O_RDONLY);
    if (fd < 0) { printf("%s:", file); perror("");  NULL; }

    // Check input file size makes sense
    int size = dfs_file_size(file);
    if ((size % DFS_TRACK_SIZE) > 0) {
        printf("%s: size of file (%d) is not a multiple of DFS track size (%d). Please check format.\n",
                file, size, DFS_TRACK_SIZE);
        return NULL;
    }
    if ((size / DFS_TRACK_SIZE) < 2) {
        printf("%s: size of file (%d) is too small to hold a catalogue. Please check format.\n",
                file, size);
        return NULL;
    }

    unsigned char *img = malloc(DFS_TRACK_SIZE);
    assert(img != NULL);
    
    int ret = read(fd, img, size);
    if (ret < 0) {
        printf("%s:", file); perror("read");
        return NULL;
        free(img);
    }
    if (ret != size) {
        printf("%s: read wrong number of bytes (%d)\n", file, ret);
        return NULL;
        free(img);
    }

    return img;
}

static void remove_nonprint_chars(char *s)
{
    for (int i = 0; i < strlen(s); i++) {
        if (!isprint(s[i])) s[i] = ' ';
    }
}

static unsigned char *offs(unsigned char *img, int sector, int offset)
{
    return img + sector * DFS_SECTOR_SIZE + offset;
}

// Extract catalogue info from a DFS image.
// See http://www.cowsarenotpurple.co.uk/bbccomputer/native/adfs.html
// for more info on DFS format.
dfs_cat_t *dfs_img_cat(unsigned char *img)
{
    assert(img != NULL);
    dfs_cat_t *cat = malloc(sizeof(dfs_cat_t));
    memcpy(cat->label, offs(img, 0, 0), 8);
    memcpy(cat->label+8, offs(img, 1, 0), 3);
    cat->label[12] = '\0'; // Null-terminate label string.
    cat->nfiles = *offs(img, 1, 5) >> 3;
    if (cat->nfiles > (DFS_SECTOR_SIZE-8)) {
        printf("warning - number of files (%d) too large.\n", cat->nfiles);
    }
    cat->nsectors = ((*offs(img, 1, 6) & 0x0f) << 8) + *offs(img, 1, 7);
    cat->boot_option = (*offs(img, 1, 6) >> 4) & 0xf;
    cat->files = calloc(cat->nfiles, sizeof(dfs_cat_file_t));
    for (int i = 0; i < cat->nfiles; i++) {
        dfs_cat_file_t *f = cat->files + i;

        // Sector 0 contains the name info in 8-byte blocks
        memcpy(f->name, offs(img, 0, 8+i*8), 7);
        f->name[7] = '\0'; // Null-terminate string
        // Filenames are padded with spaces. Null-terminate instead.
        for (int j = 0; j < 7; j++) {
            if (f->name[j] == ' ') f->name[j] = '\0';
        }
        // Dir and lock state
        f->dir = *offs(img, 0, 8+i*8 + 7);
        f->locked = (f->dir & 0x80) ? 1 : 0;
        f->dir &= 0x7f;
 
        // Sector 1 contains the addresses, lengths and locations.
        f->load_addr =
            *offs(img, 1, 8+i*8 + 0) + 
            (*offs(img, 1, 8+i*8 + 1) << 8) +
            (((*offs(img, 1, 8+i*8 + 6) >> 2) & 3) << 16);
        f->exec_addr =
            *offs(img, 1, 8+i*8 + 2) + 
            (*offs(img, 1, 8+i*8 + 3) << 8) +
            (((*offs(img, 1, 8+i*8 + 6) >> 6) & 3) << 16);
        f->size =
            *offs(img, 1, 8+i*8 + 4) + 
            (*offs(img, 1, 8+i*8 + 5) << 8) +
            (((*offs(img, 1, 8+i*8 + 6) >> 4) & 3) << 16);
        f->sector =
            *offs(img, 1, 8+i*8 + 7) + 
            (((*offs(img, 1, 8+i*8 + 6) >> 0) & 3) << 8);
    }
    return cat;
}

// A small feline legs it for the scenery.
void dfs_cat_free(dfs_cat_t *cat)
{
    free(cat->files);
    free(cat);
}

void dfs_cat_fprint(FILE *fp, dfs_cat_t *cat)
{
    assert(cat != NULL);
    assert(cat->files != NULL);
    char *label = strdup(cat->label);
    remove_nonprint_chars(label);
    fprintf(fp, "Label \"%s\", %2d tracks, boot option %2d, %2d files:\n",
            label,
            cat->nsectors/DFS_SECTORS_PER_TRACK,
            cat->boot_option,
            cat->nfiles);
    for (int i = 0; i < cat->nfiles; i++) {
        dfs_cat_file_t *f = cat->files + i;
        if (f->dir == '$') { // Don't show default dir
            fprintf(fp, "  %-7s  ", f->name);
        } else {
            fprintf(fp, "%c.%-7s  ", f->dir, f->name);
        }
        fprintf(fp, " size %6d, sector %3d, load 0x%05X, exec 0x%05X\n",
                f->size, f->sector, f->load_addr, f->exec_addr);
    }
}
