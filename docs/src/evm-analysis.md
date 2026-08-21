# EVM analysis model

VERTOX operates on deployed EVM runtime bytecode.

The reverse pipeline is:

1. decode raw or hexadecimal bytecode
2. split it into EVM instructions
3. identify potential basic-block boundaries
4. resolve directly encoded jump targets when possible
5. write text disassembly and Graphviz/JSON CFG output

Dynamic jump analysis, decompilation, symbolic execution, and ABI recovery are deliberately outside the current static engine. VERTOX avoids inventing certainty where the bytecode does not provide it.
