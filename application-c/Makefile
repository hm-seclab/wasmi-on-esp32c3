OUT_DIR = build/
CC=clang

# set the right flags to get one page of memory with just a small stack (7584B)
LDFLAGS =-Wl,--initial-memory=65536 -Wl,--max-memory=65536 -Wl,-zstack-size=7584 -Wl,--global-base=32778 -Wl,-stack-first -Wl,-no-entry -Wl,--export-all
# build a wasm project without a std library
CFLAGS =--target=wasm32 -nostdlib -O3 -flto 
OPTFLAGS = --strip-debug --strip-dwarf

SRCS = $(wildcard src/*.c)
OBJS = $(SRCS:.c=.o)

EXEC = $(OUT_DIR)out.wasm

all: optimized
	$(MAKE) clean_objs

$(EXEC): $(OBJS)
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

optimized: $(EXEC)
	wasm-opt -Oz -o $(EXEC) $(OPTFLAGS) $(EXEC)

clean:
	rm -rf $(EXEC)

clean_objs:
	rm -rf $(OBJS)
