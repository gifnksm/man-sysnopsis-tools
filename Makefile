BIN_CRATE = synopopt
LIB_CRATE = synop

DEPS_synopopt = synop

RUSTC_LINT_FLAGS = \
	-D warnings \
	-W unnecessary-qualification \
	-W non-uppercase-statics \
	-W unnecessary-typecast

RUSTC_BIN_FLAGS  = $(RUSTC_LINT_FLAGS) -L $(LIB_DIR) -O
RUSTC_LIB_FLAGS  = $(RUSTC_LINT_FLAGS) -L $(LIB_DIR) -O
RUSTC_TEST_FLAGS = --test $(RUSTC_LINT_FLAGS) -L $(LIB_DIR) -g




ALL_CRATE = $(BIN_CRATE) $(LIB_CRATE)

SRC_DIR = src
BIN_DIR = bin
LIB_DIR = lib

BIN_PATH   = $(BIN_DIR)/$(1)
STAMP_PATH = $(LIB_DIR)/lib$(1).stamp
TEST_PATH  = $(BIN_DIR)/$(1).test
TEST_TARGET  = $(addprefix test-,$(ALL_CRATE))
CHECK_TARGET = $(addprefix check-,$(ALL_CRATE))

RUSTC = rustc

.PHONY: all test check clean $$(TEST_TARGET) $$(CHECK_TARGET)

all:  $(foreach crate,$(BIN_CRATE),$(call BIN_PATH,$(crate)))
test: $(TEST_TARGET)
check: $(CHECK_TARGET)
clean:
	$(RM) $(BIN_DIR)/* $(LIB_DIR)/*

BIN_CRATE_FILE_PATH = $(SRC_DIR)/$(1)/main.rs
LIB_CRATE_FILE_PATH = $(SRC_DIR)/lib$(1)/lib.rs

BIN_TARGET_PATH  = $(call BIN_PATH,$(1))
LIB_TARGET_PATH  = $(call STAMP_PATH,$(1))
TEST_TARGET_PATH = $(call TEST_PATH,$(1))

BIN_RUSTC_CMD  = $(RUSTC) $(RUSTC_BIN_FLAGS)  $(CRATE_FILE_$(1)) -o $(2)
LIB_RUSTC_CMD  = $(RUSTC) $(RUSTC_LIB_FLAGS)  $(CRATE_FILE_$(1)) --out-dir $(LIB_DIR) && touch $(2)
TEST_RUSTC_CMD = $(RUSTC) $(RUSTC_TEST_FLAGS) $(CRATE_FILE_$(1)) -o $(2)

define CRATE_DEFINE
CRATE_FILE_$(2) := $$(call $(1)_CRATE_FILE_PATH,$(2))
SRC_FILE_$(2)   := $$(wildcard $(SRC_DIR)/$(2)/*.rs)
DEP_STAMP_$(2)  := $(foreach crate,$(DEPS_$(2)),$(call STAMP_PATH,$(crate)))

test-$(2): $$(call TEST_PATH,$(2))
check-$(2): test-$(2)
	./$$(call TEST_PATH,$(2))

$$(call $(1)_TARGET_PATH,$(2)): $$(SRC_FILE_$(2)) $$(DEP_STAMP_$(2))
	$$(call $(1)_RUSTC_CMD,$(2),$$@)
$$(call TEST_TARGET_PATH,$(2)): $$(SRC_FILE_$(2)) $$(DEP_STAMP_$(2))
	$$(call TEST_RUSTC_CMD,$(2),$$@)
endef

$(foreach crate,$(BIN_CRATE),$(eval $(call CRATE_DEFINE,BIN,$(crate))))
$(foreach crate,$(LIB_CRATE),$(eval $(call CRATE_DEFINE,LIB,$(crate))))
