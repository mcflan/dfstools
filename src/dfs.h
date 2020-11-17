/*
 * Definition of Acorn DFS filesystem.
 *
 * J.Macfarlane November 2020.
 */

#include <stdio.h>
#include <stdint.h>

// BBC Micro DFS format
#define DFS_SECTOR_SIZE             (256)
#define DFS_SECTORS_PER_TRACK       (10)
#define DFS_TRACK_SIZE              (DFS_SECTOR_SIZE * DFS_SECTORS_PER_TRACK)
#define DFS_LABEL_SIZE              (12)
#define DFS_FILENAME_LEN            (7)

typedef struct dfs_cat_file_s {
    char name[DFS_FILENAME_LEN+1];
    char dir;
    int locked;
    uint32_t load_addr; 
    uint32_t exec_addr; 
    uint32_t size; 
    uint16_t sector; 
} dfs_cat_file_t;

typedef struct dfs_cat_s {
    char label[13];
    int nfiles;
    dfs_cat_file_t *files;
    int nsectors;
    int boot_option;
} dfs_cat_t;

int dfs_file_size(char *filename);
unsigned char *dfs_open_image(char *file);
dfs_cat_t *dfs_img_cat(unsigned char *img);
void dfs_cat_free(dfs_cat_t *cat);
void dfs_cat_fprint(FILE *fp, dfs_cat_t *cat);
