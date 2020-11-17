/*
 * split.c
 *
 * Split a double-sided .dsd BBC Disk image file into two single-sided (.ssd)
 * ones.
 *
 * J.Macfarlane November 2020
 */


#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <assert.h>

#include "dfs.h"

int main(int argc, char **argv)
{
    if (argc <= 3) {
        printf("usage: %s input_file side1_output_file side2_output_file\n", argv[0]);
        return 1;
    }

    char *input_file= argv[1];
    char *side1_file = argv[2];
    char *side2_file = argv[3];

    // Check we can open all the files.
    int input_fd = open(input_file, O_RDONLY);
    if (input_fd < 0) { fprintf(stderr, "%s:", input_file); perror(""); return 1; }
    int side1_fd = open(side1_file, O_WRONLY | O_CREAT, 0644);
    if (side1_fd < 0) { fprintf(stderr, "%s:", side1_file); perror(""); return 1; }
    int side2_fd = open(side2_file, O_WRONLY | O_CREAT, 0644);
    if (side2_fd < 0) { fprintf(stderr, "%s:", side2_file); perror(""); return 1; }

    // Check input file size makes sense
    int input_size = dfs_file_size(input_file);
    if (input_size % (DFS_TRACK_SIZE * 2) > 0) {
        fprintf(stderr, "%s: size of file (%d) is not a multiple of 2x track size (%d). Please check format.\n",
                input_file, input_size, DFS_TRACK_SIZE);
        return 1;
    }

    // n_tracks while be twice number of tracks per side.
    int n_tracks = input_size / DFS_TRACK_SIZE;

    // Make a buffer for getting tracks
    unsigned char *trackbuf = malloc(DFS_TRACK_SIZE);
    assert(trackbuf != NULL);

    // Read tracks and write them alternately to the two output files.
    for (int i = 0; i < n_tracks; i++) {
        int ret = read(input_fd, trackbuf, DFS_TRACK_SIZE);
        if (ret < 0) { printf("%s:", input_file); perror("read"); return 1; }
        if (ret != DFS_TRACK_SIZE) {
            fprintf(stderr, "%s: read wrong number of bytes at track %d\n", input_file, i/2);
            return 1;
        }
        if ((i & 1) == 0) {
            write(side1_fd, trackbuf, DFS_TRACK_SIZE);
        } else {
            write(side2_fd, trackbuf, DFS_TRACK_SIZE);
        }
    }
    close(input_fd);
    close(side1_fd);
    close(side2_fd);
    return 0;
}
