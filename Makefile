RUSTCRATES = synopfmt synopexpand cmdutil synop

synopfmt_CRATE_DEPS    += cmdutil synop
synopexpand_CRATE_DEPS += cmdutil synop

RUSTCFLAGS += \
	-D warnings \
	-W unnecessary-qualification \
	-W non-uppercase-statics \
	-W unnecessary-typecast

RUSTBINDIR = bin
RUSTLIBDIR = lib

export RUST_TEST_TASKS = 1
include rust-mk/rust.mk
