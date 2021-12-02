OUT_DIR = build/
CC=clang++

comma = ,
# set the right flags to get one page of memory with just a small stack (7584B)
LDFLAGS =$(addprefix -Wl$(comma),--initial-memory=65536 --max-memory=65536 -zstack-size=7584 --global-base=32778 -stack-first -no-entry --export-all)
# build a wasm project without a std library
CXXFLAGS =-std=c++11 -fno-rtti --target=wasm32 -nostdlib -O3 -flto 
OPTFLAGS =--strip-debug --strip-dwarf

SRC = src/main.cpp
DEPS = $(filter-out $(SRC),$(wildcard src/*.cpp))
OBJS = $(SRCS:.cpp=.o)

EXEC = $(OUT_DIR)out.wasm

all: optimized
	$(MAKE) clean_objs

$(EXEC): $(OBJS)
	$(CC) $(CXXFLAGS) -o $@ $^ $(addprefix --include ,$(DEPS)) $(LDFLAGS) $(SRC) 

optimized: $(EXEC)
	wasm-opt -Oz -o $(EXEC) $(OPTFLAGS) $(EXEC)

clean:
	rm -rf $(EXEC)

clean_objs:
	rm -rf $(OBJS)
