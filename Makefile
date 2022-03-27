ARCH = riscv64
TARGET = riscv64gc-unknown-none-elf

all:
	mkdir target -p
	cd crates/nekos-initproc && cargo build
	cd kernel && cargo build --profile dev && cp target/$(TARGET)/debug/main ../target/make.elf
	rust-objcopy \
		--strip-all --remove-section=.uninit \
		-B binary-architecture=$(ARCH) -O binary \
		target/make.elf target/make.bin

clean:
	cd crates/nekos-initproc && cargo clean
	cd kernel && cargo clean
	rm -rf target/*

run: all
	qemu-system-$(ARCH) \
		-global virtio-mmio.force-legacy=false \
		-machine virt -bios default \
		-nographic -smp cpus=8 -m 512M \
		-kernel target/make.bin
