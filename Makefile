$(HOME)/.cargo/bin/evcxr:
	@cargo install evcxr_repl

evcxr: $(HOME)/.cargo/bin/evcxr
	evcxr

$(HOME)/.cargo/bin/oxischeme:
	@cargo install oxischeme

oxischeme: $(HOME)/.cargo/bin/oxischeme
	oxischeme
