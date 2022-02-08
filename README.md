# SIC-XE-assembler
SIC/XE two pass assembler written in Rust

## Usage
- move into folder "assembler"
- run with: ```cargo run "<path to .asm file>"```

Example:
    ```cargo run "../asm_files/isort.asm"```

.obj files will be created inside assembler folder.

Run instruction compiles and runs code, you could of course compile and run separately.

Errors in asm code are reported with line number and problem message.

## Executing .obj files

You can use my [SIC/XE simulator](https://github.com/blaz-r/SIC-XE-simulator) to execute .obj files, or you can use [SicTools](https://github.com/jurem/SicTools) that is more advanced.

## Supported functions
- almost all instructions
- directives START, END, ORG and EQU
- directives BASE and NOBASE
- symbol resolution
- object code (.obj) generation with H, E, T and simplified M records
- nice output of combined object and assembly code (.lst)
- arbitrary expressions with +, -, * and / in EQU directive

### Contributing

Assembler should work fine in most cases, but some bugs surely exist in the code due it being quite a large project. If you find any, open an issue and/or pull request :)