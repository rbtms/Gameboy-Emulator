import os
from random import randint

# Size in bytes of each opcode
BYTES_SIZE_INSTRS = [
  # 0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
    1, 3, 1, 1, 1, 1, 2, 1, 3, 1, 1, 1, 1, 1, 2, 1, # 0
    1, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, # 1
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, # 2
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, # 3
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, # 4
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, # 5
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, # 6 
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, # 7
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, # 8
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, # 9
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, # a
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, # b
    1, 1, 3, 3, 3, 1, 2, 1, 1, 1, 3, 1, 3, 3, 2, 1, # c
    1, 1, 3, 0, 3, 1, 2, 1, 1, 1, 3, 0, 3, 0, 2, 1, # d
    2, 1, 1, 0, 0, 1, 2, 1, 2, 1, 3, 0, 0, 0, 2, 1, # e
    2, 1, 1, 1, 0, 1, 2, 1, 2, 1, 3, 1, 0, 0, 2, 1, # f
]

BYTES_SIZE_INSTR_CB = 2 # Every 0xCB prefixed opcode is 2 byte long

# Opcodes which modify memory at (BC)
MOD_MEM_RR_OPCODES = [
    0x02, # LD (BC), A
    0x12, # LD (DE), A
    0x77, # LD (HL), A
    0x22, # LD (HL+), A
    0x32, # LD (HL-), A
]

# Opcodes which modify memory at (HL)
MOD_MEM_HL_R = [
    0x70, 0x71, 0x72, 0x73, # LD (HL), r
]

# Unsafe opcodes which modify memory
MOD_MEM_HL_OPCODES = [
    0x74, 0x75, # LD (HL) H, LD (HL) L. Not safe since it would point to a different direction
    0x36, # LD (HL), u8. Not safe since there is no way to set it
    0x34, # INC (HL). Not safe since it could modify the next instruction
    0x35, # DEC (HL). ""
]

# Opcodes which modify memory as an offset of 0xFF00
MOD_MEM_OFFSET_OPCODES = [
    0xE0, # FF00+u8
    0xE2, # FF00+C

]

# Opcodes which modify memory pointed by a u16
MOD_MEM_IMM_OPCODES = [
    0x08, # LD (u16), SP
    0xEA  # LD (u16), A
]

# Opcodes which modify memory
MOD_MEM_OPCODES = MOD_MEM_HL_OPCODES + MOD_MEM_OFFSET_OPCODES + MOD_MEM_IMM_OPCODES

# Opcodes which modify SP
MOD_SP_OPCODES = [
    0x31, # LD SP
    0x33, # INC SP
    0x3B, # DEC SP
    0xE8, # ADD SP, i8
    0xF9, # LD SP, HL
]

# Opcodes which modify the stack
MOD_STACK_OPCODES = [
    0xC5, # PUSH BC
    0xD5, # PUSH DE
    0xE5, # PUSH HL
    0xF5, # PUSH AF
    0xC1, # POP BC
    0xD1, # POP DE
    0xE1, # POP HL
    0xF1  # POP AF
]

JUMP_OPCODES = [
    0x18, # JR i8
    0x20, # JR NZ, i8
    0x28, # JR Z,  i8
    0x30, # JR NC, i8
    0x38, # JR C,  i8
    0xC2, # JP NZ, u16
    0xC3, # JP u16
    0xCA, # JP Z, u16
    0xD2, # JP NC, u16
    0xDA, # JP C, u16
    0xE9, # JP HL
]

CALL_OPCODES = [
    0xC4, # CALL NZ, u16
    0xCC, # CALL Z, u16
    0xCD, # CALL u16
    0xD4, # CALL NC, u16
    0xDC, # CALL C, u16
]

# Opcodes to not include. HALT, STOP, EI, DI, RETI, RET, RET NC, RET NZ
OMIT_OPCODES = [
    0x10, # STOP
    0x76, # HALT
    0xC0, # RET NC
    0xC8, # RET Z
    0xC9, # RET
    0xD0, # RET NZ
    0xD8, # RET C
    0xD9, # RETI
    0xF3, # DI
    0xF8, # EI
]

# Invalid instructions
INVALID_OPCODES = [
    0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD
]

RST_OPCODES = [0xC0, 0xC7, 0xCF, 0xD7, 0xDF, 0xE7, 0xEF, 0xF7, 0xFF]

RST_ADDR = [0x00, 0x08, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38]
OPCODE_RET = 0xC9

# Addresses used for interrupts
INTERRUPT_ADDR = [0x40, 0x48, 0x50, 0x58, 0x60]


# Generate a random opcode
def get_rand_opcode_val():
    opcode = randint(0, len(BYTES_SIZE_INSTRS)-1)
    
    while (opcode in INVALID_OPCODES) or (opcode in OMIT_OPCODES)\
       or (opcode in MOD_MEM_OPCODES)\
       or (opcode == 0xCB):
        opcode = randint(0, len(BYTES_SIZE_INSTRS)-1)

    return opcode

# Generate a u8
def gen_imm_u8():
    return [randint(0, 0xFF)]

# Generate a u16
def gen_imm_u16():
    val = randint(0, 0xFFFF)
    # lo, hi
    return [val&0xFF, val>>8]

def gen_rand_cb_opcode():
    opcode = randint(0, 0xFF) # 0x00 ~ 0xFF

    # Opcodes which modify memory
    while (opcode&0x0F) in (0x06, 0x0E):
        opcode = randint(0, 0xFF)
    
    return [0xCB, opcode], BYTES_SIZE_INSTR_CB

def gen_rand_opcode(pos):
    opcode = get_rand_opcode_val()
    size   = BYTES_SIZE_INSTRS[opcode]
    program = []

    if opcode in JUMP_OPCODES:
        # If it's a relative jump, jump to the same place, since it's 2 bytes
        # ahead (1 after fetching the opcode, 1 after fetching the i8)
        if opcode < 0x40: #JR i8
            program = [opcode, 0]
        # If it uses HL, give it the value of the next position (2 instructions after)
        elif opcode == 0xE9: # JP HL
            program = [0x21, (pos+4)&0xFF, (pos+4)>>8] # LD HL pos+1
            program += [opcode] # JP HL
            
            size += 3 # From the first instruction
        # If it has a u16 imm value, load the next position in memory
        else: # JP u16
            program = [opcode, (pos+3)&0xFF, (pos+3)>>8]
    # CALL: Point call address to 0 where there is a RET
    elif opcode in CALL_OPCODES:
        program = [opcode, 0, 0]
    # MOD SP: Save SP and restore back afterwards
    elif opcode in MOD_SP_OPCODES:
        # Save SP
        program = [0xF8, 0] # LD HL, SP+0
        size += 2

        # Add opcode
        program += [opcode]

        # LD SP, u16
        if opcode == 0x31:
            program += [0, 0]
        # ADD SP, i8
        if opcode == 0xE8:
            program += [0]

        # Load SP back
        program += [0xF9] # LD SP, HL
        size += 1
    # If the opcode pushes or pops from the stack, append the reverse operation
    elif opcode in MOD_STACK_OPCODES:
        program = [opcode]

        # Push rr
        if opcode&0x0F == 0x05:
            program += [opcode&0xF0 | 0x01] # Pop rr
        # Pop rr
        else:
            program += [opcode&0xF0 | 0x05] # Push rr
        
        size += 1
    # Replace the value in memory by the same value
    elif opcode in MOD_MEM_RR_OPCODES:
        if opcode == 0x02: # LD (BC), A
            program  = [0x0A] # PUSH A, (BC)
            program += [opcode]
        elif opcode == 0x12: # LD (DE), A
            program  = [0x1A] # PUSH A, (DE)
            program += [opcode]
        else: # LD (HL/HL+/HL-), A
            program  = [0x7E] # A, (HL)
            program += [opcode]

        size += 1
    elif opcode in MOD_MEM_HL_R: # LD (HL), r
        oposite_opcodes = [0x46, 0x4E, 0x56, 0x5E, 0x66, 0x6E]

        program  = [oposite_opcodes[opcode&0x0F]]
        program += [opcode]

        #print(list(map(hex, program)))

        size += 1
    else:
        program = [opcode]

        if   size == 2: program += gen_imm_u8()
        elif size == 3: program += gen_imm_u16()

    return program, size

# Generate a valid program
def gen_program():
    program = [0x00 for _ in range(0x150)] # Header
    pos = 0x150 # Keep track for jumps

    # Use Test ROM
    program[0x147] = 0xFF
    # Jump to program start
    program[0x100] = 0xC3
    program[0x101] = 0x150&0xFF
    program[0x102] = 0x150>>8

    # RST opcodes
    for addr in RST_ADDR + INTERRUPT_ADDR:
        program[addr] = OPCODE_RET

    # Generate program until 0xFFF0
    while len(program) < 0xFF00:
        is_cb = randint(0, 1) == 0
        
        byte_arr, size = gen_rand_cb_opcode() if is_cb else gen_rand_opcode(pos)
        program += byte_arr
        pos += size

    # Set last positions as STOP. Not just last position because
    # some of the last places could get overwritten by the stack
    while len(program) < 0x10000:
        program.append(0x10)
    
    return program

# Write a program to disk
def write_rom(program):
    filename = f'roms/benchmark/benchmark_rom_{randint(0, 1000)}.gb'
    #filename = f'roms/benchmark/benchmark_rom.gb'

    with open(filename, 'wb') as f:
        f.write(bytes(program))

# Regenerate all test ROMs
os.system('rm roms/benchmark/*')
for i in range(100):
    program = gen_program()
    write_rom(program)
