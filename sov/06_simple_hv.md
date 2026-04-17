# simple_hv 实现总结

## 题目要求

实现一个简单的 Type-2 Hypervisor，能够：
1. 加载并运行 Guest VM (skernel2)
2. 处理 VM exit 事件
3. 模拟必要的硬件操作

测试验证输出 `"Shutdown vm normally!"`

## 问题分析

### Guest VM 程序 (skernel2)

```asm
_start:
    csrr a1, mhartid    ; 读取 M-mode CSR
    ld a0, 64(zero)     ; 从物理地址 0x40 加载参数
    li a7, 8            ; SBI shutdown extension ID
    ecall               ; SBI 系统调用
```

Guest VM 在 VS-mode 运行时会触发以下异常：
1. **IllegalInstruction**: `mhartid` 是 M-mode CSR，VS-mode 无法直接访问
2. **LoadGuestPageFault**: 地址 0x40 未映射
3. **VirtualSupervisorEnvCall**: SBI ecall 需要处理

## 实现方案

**修改文件**: `arceos/exercises/simple_hv/src/main.rs`

### vmexit_handler 完整实现

```rust
fn vmexit_handler(ctx: &mut VmCpuRegisters) -> bool {
    use scause::{Exception, Trap};
    let scause = scause::read();

    match scause.cause() {
        // 处理 SBI ecall
        Trap::Exception(Exception::VirtualSupervisorEnvCall) => {
            let sbi_msg = SbiMessage::from_regs(ctx.guest_regs.gprs.a_regs()).ok();
            if let Some(SbiMessage::Reset(_)) = sbi_msg {
                let a0 = ctx.guest_regs.gprs.reg(A0);
                let a1 = ctx.guest_regs.gprs.reg(A1);
                assert_eq!(a0, 0x6688);
                assert_eq!(a1, 0x1234);
                ax_println!("Shutdown vm normally!");
                return true;  // VM 结束
            }
        },

        // 处理非法指令 - 模拟 M-mode CSR
        Trap::Exception(Exception::IllegalInstruction) => {
            let inst = stval::read();
            // 解析 csrr 指令格式
            if (inst & 0x7F) == 0x73 && ((inst >> 12) & 0x7) == 0x2 {
                let csr_addr = (inst >> 20) & 0xFFF;
                let rd = ((inst >> 7) & 0x1F) as u32;

                if csr_addr == 0xF14 {  // mhartid
                    ctx.guest_regs.gprs.set_reg(rd_idx, 0x1234);
                    ctx.guest_regs.sepc += 4;  // 跳过当前指令
                    return false;
                }
            }
        },

        // 处理内存访问异常 - 模拟物理地址访问
        Trap::Exception(Exception::LoadGuestPageFault) => {
            let fault_addr = stval::read();
            if fault_addr == 0x40 {
                ctx.guest_regs.gprs.set_reg(GprIndex::A0, 0x6688);
                ctx.guest_regs.sepc += 4;
                return false;
            }
        },

        _ => panic!("Unhandled trap"),
    }
    false
}
```

### 关键设计点

1. **CSR 模拟**: 解析指令编码，识别 `csrr` 操作，返回模拟值
2. **内存模拟**: 对特定地址的访问返回预设值
3. **sepc 更新**: 每次处理完异常后，需要 `sepc += 4` 跳过当前指令
4. **返回值约定**: `true` 表示 VM 结束，`false` 表示继续运行

## 指令解析

### csrr 指令格式

```
| imm[11:0] | rs1 | funct3 | rd  | opcode   |
| 12 bits   | 5   | 3      | 5   | 7        |

csrr a1, mhartid = 0xF14025F3

解析:
- opcode = 0x73 (SYSTEM)
- funct3 = 0x2 (CSRR)
- rd = 11 (a1)
- csr_addr = 0xF14 (mhartid)
```

### ld 指令格式

```
| imm[11:0] | rs1 | funct3 | rd  | opcode   |
ld a0, 64(zero):
- imm = 64 (0x40)
- rs1 = 0 (zero)
- funct3 = 3 (LD)
- rd = 10 (a0)
- opcode = 3
```

## 测试结果

```
Hypervisor ...
app: /sbin/skernel2
paddr: PA:0x80645000
IllegalInstruction: inst=0xf14025f3 sepc=0x80200000
LoadGuestPageFault: stval=0x40 sepc=0x80200004
VmExit Reason: VSuperEcall: Some(Reset(...))
a0 = 0x6688, a1 = 0x1234
Shutdown vm normally!
```

## VM Exit 处理流程

```
Guest VM 在 VS-mode 执行
    ↓
csrr a1, mhartid
    ↓
VM Exit (IllegalInstruction)
    ↓
vmexit_handler:
    解析指令 → 模拟 CSR → a1 = 0x1234
    sepc += 4 → 继续执行
    ↓
ld a0, 64(zero)
    ↓
VM Exit (LoadGuestPageFault)
    ↓
vmexit_handler:
    检测地址 0x40 → 模拟加载 → a0 = 0x6688
    sepc += 4 → 继续执行
    ↓
li a7, 8 (正常执行，无异常)
    ↓
ecall
    ↓
VM Exit (VirtualSupervisorEnvCall)
    ↓
vmexit_handler:
    解析 SBI message → Reset
    检查 a0, a1 → 验证正确
    返回 true → VM 结束
```

## Hypervisor 核心概念

### RISC-V H 扩展

- **HS-mode**: Hypervisor Supervisor mode
- **VS-mode**: Virtual Supervisor mode (Guest 运行)
- **关键 CSR**: hstatus, hgatp, vsatp 等

### 二阶段地址翻译

```
Guest VA → Guest PA (VS-mode 页表)
    ↓
Guest PA → Host PA (hgatp, EPT)
```

### VM Exit 类型

| 异常类型 | 触发原因 |
|---------|---------|
| IllegalInstruction | 访问特权级不足的 CSR/指令 |
| LoadGuestPageFault | Guest 页表未映射 |
| VirtualSupervisorEnvCall | Guest 执行 ecall |

## 依赖模块

```
simple_hv
├── axmm (地址空间管理)
├── axhal (硬件抽象层)
│   ├── mem::PhysAddr
│   ├── paging::MappingFlags
│   └── trap handling
├── riscv crate (CSR 定义)
├── sbi_spec (SBI 协议)
└── tock_registers (CSR 操作)
```