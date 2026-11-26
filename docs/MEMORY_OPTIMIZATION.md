# Memory Optimization (2 GB Host Target)

ExR x86 targets hosts with 2 GiB physical RAM without sacrificing guest execution correctness or memory safety.

## Implemented Optimizations

1. **TCG Code-Gen Buffer Auto-Sizing (`tcg/region.c`)**
   - Upstream QEMU defaults to `phys_mem / 8` up to `1 GiB` (256 MiB on a 2 GiB host, 512 MiB on 4 GiB).
   - `tcg_region_init()` now caps the default TCG translation buffer to:
     - `32 MiB` when `qemu_get_host_physmem() <= 2560 MiB`
     - `64 MiB` when `qemu_get_host_physmem() <= 4608 MiB`
   - Explicit `engine.tb-size` or `engine.profile: "low-mem-2gb"` / `"minimal"` overrides this on any host.

2. **Lazy 4 KiB Guest RAM Faulting (`system/physmem.c`)**
   - `ram_block_add()` applies `QEMU_MADV_NOHUGEPAGE` when host physical RAM is `<= 3 GiB` (and in `tcg_region_init()` when `<= 4.5 GiB`).
   - Prevents Linux Transparent Huge Pages (THP) from forcing 2 MiB physical allocations on sparse 4 KiB guest page touches and avoids THP compaction stalls under memory pressure.

3. **SoftMMU Dynamic TLB Cap (`accel/tcg/cputlb.c`)**
   - `tlb_mmu_resize_locked()` caps dynamic TLB growth to `14` bits (16,384 entries = 512 KiB per MMU index) on `<= 3 GiB` hosts instead of `20` bits (32 MiB per MMU index), preventing multi-vCPU TLB bloat.

4. **`.exrimg` L2 Cache & Block Reclamation (`exr/cli.c`)**
   - `low-mem-2gb` and `minimal` profiles cap `.exrimg` (`qcow2`) L2 metadata cache to `1 MiB` (`4 MiB` for standard).
   - Enables `discard=unmap` and `detect-zeroes=unmap` so zeroed/freed guest blocks are reclaimed on the host.
   - Offloads disk I/O to a dedicated `iothread` (`exr-iothread0`) with `aio=threads`.

5. **Dynamic Memory Ballooning**
   - Setting `engine.balloon: true` (or `exr_config_set_balloon(cfg, true)`) attaches `virtio-balloon-pci` so the host/frontend panel can reclaim unused guest pages at runtime.
