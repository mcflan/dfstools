
CFLAGS=-Wall

TARGETS=split dfs_cat dfs_extract

all: $(TARGETS)

split: split.o dfs.o

dfs_cat: dfs_cat.o dfs.o

dfs_extract: dfs_extract.o dfs.o

clean:
	rm -f *.o $(TARGETS)
