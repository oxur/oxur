default: build

$(HOME)/.cargo/bin/evcxr:
	@cargo install evcxr_repl

evcxr: $(HOME)/.cargo/bin/evcxr
	evcxr

$(HOME)/.cargo/bin/oxischeme:
	@cargo install oxischeme

oxischeme: $(HOME)/.cargo/bin/oxischeme
	oxischeme

build:
	@cargo build
	@rm bin/*
	@cargo install --path . --root .

rebuild:
	@cargo clean
	$(MAKE) build
