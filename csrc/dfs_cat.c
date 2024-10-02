/*
 * dfs_cat.c
 *
 * Display catalogue data of an Acorn DFS image.
 *
 * J.Macfarlane November 2020
 */

#include <stdio.h>
#include <stdlib.h>

#include "dfs.h"

int main(int argc, char **argv)
{
    if (argc <= 1) {
        printf("usage: %s image.ssd [...]\n", argv[0]);
        return 1;
    }

    for (int i = 1; i < argc; i++) {
        char *img_file = argv[i];
        unsigned char *img = dfs_open_image(img_file);
        if (img == NULL) { return 1; }
        dfs_cat_t *cat = dfs_img_cat(img);
        dfs_cat_fprint(stdout, cat);
        dfs_cat_free(cat);
    }

    return 0;
}
