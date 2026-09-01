/**
 * Sher Language Web Execution Engine v0.1.0
 * Pure JavaScript client-side implementation of the Sher interpreter.
 */

class SherEngine {
    constructor() {
        this.outputCallback = null;
        this.virtualFs = {};
    }

    setOutputCallback(cb) {
        this.outputCallback = cb;
    }

    emitPrint(args) {
        const text = args.map(a => this.formatValue(a)).join(' ');
        if (this.outputCallback) {
            this.outputCallback('out', text);
        } else {
            console.log(text);
        }
    }

    formatValue(v) {
        if (v === null || v === undefined) return 'null';
        if (typeof v === 'boolean') return v ? 'true' : 'false';
        if (typeof v === 'number') return v.toString();
        if (typeof v === 'string') return v;
        if (Array.isArray(v)) {
            return '[' + v.map(item => this.formatValue(item)).join(', ') + ']';
        }
        if (v.__is_tuple) {
            return '(' + v.items.map(item => this.formatValue(item)).join(', ') + ')';
        }
        if (v.__is_enum) {
            return `${v.enum_name}::${v.variant}`;
        }
        if (v.__is_struct) {
            const fields = Object.entries(v.fields)
                .map(([k, val]) => `${k}: ${this.formatValue(val)}`)
                .join(', ');
            return `${v.struct_name} { ${fields} }`;
        }
        if (v.__is_map) {
            const entries = v.entries
                .map(([k, val]) => `${this.formatValue(k)}: ${this.formatValue(val)}`)
                .join(', ');
            return `{${entries}}`;
        }
        return String(v);
    }

    run(sourceCode) {
        try {
            const lexer = new SherLexer(sourceCode);
            const tokens = lexer.tokenize();
            const parser = new SherParser(tokens, sourceCode);
            const ast = parser.parse();
            const interpreter = new SherInterpreter(ast, sourceCode, this);
            interpreter.execute();
            return { success: true };
        } catch (err) {
            return {
                success: false,
                error: err.message,
                formatted: err.formatted || err.message,
                line: err.line || 1,
                col: err.col || 1
            };
        }
    }
}

// ==================== LEXER ====================
class SherLexer {
    constructor(source) {
        this.source = source;
        this.tokens = [];
        this.pos = 0;
        this.line = 1;
        this.col = 1;
    }

    peek() {
        return this.pos < this.source.length ? this.source[this.pos] : null;
    }

    peekNext() {
        return this.pos + 1 < this.source.length ? this.source[this.pos + 1] : null;
    }

    advance() {
        const ch = this.peek();
        this.pos++;
        if (ch === '\n') {
            this.line++;
            this.col = 1;
        } else {
            this.col++;
        }
        return ch;
    }

    tokenize() {
        while (this.pos < this.source.length) {
            const ch = this.peek();
            const startLine = this.line;
            const startCol = this.col;

            if (ch === ' ' || ch === '\t' || ch === '\r' || ch === '\n') {
                this.advance();
                continue;
            }

            // Comments
            if (ch === '/' && this.peekNext() === '/') {
                while (this.peek() && this.peek() !== '\n') {
                    this.advance();
                }
                continue;
            }

            // Numbers
            if (ch >= '0' && ch <= '9') {
                let numStr = '';
                let isFloat = false;
                while (this.peek() && ((this.peek() >= '0' && this.peek() <= '9') || (this.peek() === '.' && !isFloat && this.peekNext() >= '0' && this.peekNext() <= '9'))) {
                    if (this.peek() === '.') isFloat = true;
                    numStr += this.advance();
                }
                const val = isFloat ? parseFloat(numStr) : parseInt(numStr, 10);
                this.tokens.push({ type: isFloat ? 'FLOAT' : 'INT', value: val, lexeme: numStr, line: startLine, col: startCol });
                continue;
            }

            // Strings
            if (ch === '"') {
                this.advance(); // consume opening "
                let str = '';
                while (this.peek() && this.peek() !== '"') {
                    if (this.peek() === '\\') {
                        this.advance();
                        const esc = this.advance();
                        if (esc === 'n') str += '\n';
                        else if (esc === 't') str += '\t';
                        else if (esc === 'r') str += '\r';
                        else if (esc === '"') str += '"';
                        else if (esc === '\\') str += '\\';
                        else str += esc;
                    } else {
                        str += this.advance();
                    }
                }
                if (this.peek() === '"') this.advance();
                this.tokens.push({ type: 'STRING', value: str, lexeme: `"${str}"`, line: startLine, col: startCol });
                continue;
            }

            // Chars
            if (ch === '\'') {
                this.advance();
                let charVal = '';
                if (this.peek() === '\\') {
                    this.advance();
                    const esc = this.advance();
                    charVal = esc === 'n' ? '\n' : esc === 't' ? '\t' : esc;
                } else {
                    charVal = this.advance();
                }
                if (this.peek() === '\'') this.advance();
                this.tokens.push({ type: 'CHAR', value: charVal, lexeme: `'${charVal}'`, line: startLine, col: startCol });
                continue;
            }

            // Dot and DotDot (0..5)
            if (ch === '.') {
                this.advance();
                if (this.peek() === '.') {
                    this.advance();
                    this.tokens.push({ type: '..', lexeme: '..', line: startLine, col: startCol });
                } else {
                    this.tokens.push({ type: '.', lexeme: '.', line: startLine, col: startCol });
                }
                continue;
            }

            // Colons
            if (ch === ':') {
                this.advance();
                if (this.peek() === ':') {
                    this.advance();
                    this.tokens.push({ type: '::', lexeme: '::', line: startLine, col: startCol });
                } else {
                    this.tokens.push({ type: ':', lexeme: ':', line: startLine, col: startCol });
                }
                continue;
            }

            // Operators & Delimiters
            const twoChar = ch + (this.peekNext() || '');
            if (['==', '!=', '<=', '>=', '+=', '-=', '*=', '/=', '%=', '&&', '||', '++'].includes(twoChar)) {
                this.advance();
                this.advance();
                this.tokens.push({ type: twoChar, lexeme: twoChar, line: startLine, col: startCol });
                continue;
            }

            if ('+-*/%<>=!(){}[],;'.includes(ch)) {
                this.advance();
                this.tokens.push({ type: ch, lexeme: ch, line: startLine, col: startCol });
                continue;
            }

            // Identifiers and Keywords
            if ((ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || ch === '_') {
                let id = '';
                while (this.peek() && ((this.peek() >= 'a' && this.peek() <= 'z') || (this.peek() >= 'A' && this.peek() <= 'Z') || (this.peek() >= '0' && this.peek() <= '9') || this.peek() === '_')) {
                    id += this.advance();
                }

                const keywords = [
                    'func', 'let', 'var', 'struct', 'enum', 'map', 'import', 'if', 'else',
                    'while', 'for', 'in', 'return', 'break', 'continue', 'print',
                    'true', 'false', 'null',
                    'int8', 'int16', 'int26', 'int32', 'int64', 'int',
                    'float8', 'float16', 'float32', 'float64', 'float',
                    'char', 'string', 'str', 'bool', 'boolean', 'void', 'any'
                ];

                if (keywords.includes(id)) {
                    this.tokens.push({ type: id.toUpperCase(), value: id, lexeme: id, line: startLine, col: startCol });
                } else {
                    this.tokens.push({ type: 'IDENTIFIER', value: id, lexeme: id, line: startLine, col: startCol });
                }
                continue;
            }

            this.advance();
        }

        this.tokens.push({ type: 'EOF', lexeme: '', line: this.line, col: this.col });
        return this.tokens;
    }
}

// ==================== PARSER ====================
class SherParser {
    constructor(tokens, source) {
        this.tokens = tokens;
        this.source = source;
        this.current = 0;
    }

    peek() { return this.tokens[this.current]; }
    previous() { return this.tokens[this.current - 1]; }
    isAtEnd() { return this.peek().type === 'EOF'; }

    check(type) {
        if (this.isAtEnd()) return false;
        return this.peek().type === type;
    }

    match(...types) {
        for (const t of types) {
            if (this.check(t)) {
                this.advance();
                return true;
            }
        }
        return false;
    }

    advance() {
        if (!this.isAtEnd()) this.current++;
        return this.previous();
    }

    consume(type, message, hint) {
        if (this.check(type)) return this.advance();
        this.error(this.peek(), message, hint);
    }

    error(token, message, hint) {
        const line = token.line || 1;
        const col = Math.max(token.col || 1, 1);
        const lines = this.source.split('\n');
        const codeLine = lines[line - 1] || '';
        
        const lineStr = String(line);
        const gutterWidth = Math.max(lineStr.length, 1);
        const emptyGutter = ' '.repeat(gutterWidth);
        const padding = ' '.repeat(col - 1);

        const formatted = `error[SyntaxError]: ${message}\n  --> main.sr:${line}:${col}\n ${emptyGutter}|\n ${lineStr} | ${codeLine}\n ${emptyGutter}| ${padding}^\n ${emptyGutter}|\n  = help: ${hint || 'Check syntax at this position'}\n`;
        const err = new Error(message);
        err.formatted = formatted;
        err.line = line;
        err.col = col;
        throw err;
    }

    parse() {
        const statements = [];
        while (!this.isAtEnd()) {
            statements.push(this.declaration());
        }
        return statements;
    }

    declaration() {
        if (this.match('STRUCT')) return this.structDef();
        if (this.match('ENUM')) return this.enumDef();
        if (this.match('FUNC')) return this.functionDef();
        if (this.match('VAR', 'LET')) return this.varDecl(this.previous().type === 'LET');
        if (this.match('IMPORT')) return this.importStmt();
        return this.statement();
    }

    importStmt() {
        let moduleName = '';
        if (this.match('<')) {
            moduleName = this.advance().lexeme;
            this.consume('>', 'Expected \'>\' after module name', 'Use syntax: import <math>');
        } else {
            moduleName = this.advance().value;
        }
        this.match(';'); // optional
        return { type: 'Import', module: moduleName };
    }

    structDef() {
        const name = this.consume('IDENTIFIER', 'Expected struct name').value;
        this.consume('{', 'Expected \'{\' after struct name');
        const fields = [];
        while (!this.check('}') && !this.isAtEnd()) {
            const ftype = this.parseType();
            this.consume(':', 'Expected \':\' after field type');
            const fname = this.consume('IDENTIFIER', 'Expected field name').value;
            this.consume(';', 'Expected \';\' after field definition');
            fields.push({ name: fname, type: ftype });
        }
        this.consume('}', 'Expected \'}\' after struct fields');
        this.match(';');
        return { type: 'StructDef', name, fields };
    }

    enumDef() {
        const name = this.consume('IDENTIFIER', 'Expected enum name').value;
        this.consume('{', 'Expected \'{\' after enum name');
        const variants = [];
        while (!this.check('}') && !this.isAtEnd()) {
            const vname = this.consume('IDENTIFIER', 'Expected enum variant name').value;
            let val = null;
            if (this.match('=')) {
                val = this.expression();
            }
            variants.push({ name: vname, value: val });
            this.match(',', ';');
        }
        this.consume('}', 'Expected \'}\' after enum variants');
        this.match(';');
        return { type: 'EnumDef', name, variants };
    }

    functionDef() {
        const name = this.consume('IDENTIFIER', 'Expected function name').value;
        this.consume('(', 'Expected \'(\' after function name');
        const params = [];
        if (!this.check(')')) {
            do {
                let ptype = 'any';
                let pname = '';
                if (this.checkType()) {
                    ptype = this.parseType();
                    if (this.match(':')) {
                        // consumed ':'
                    }
                    pname = this.consume('IDENTIFIER', 'Expected parameter name').value;
                } else {
                    pname = this.consume('IDENTIFIER', 'Expected parameter name').value;
                    if (this.match(':')) {
                        if (this.checkType()) {
                            ptype = this.parseType();
                        }
                    }
                }
                params.push({ name: pname, type: ptype });
            } while (this.match(',', ';'));
        }
        this.consume(')', 'Expected \')\' after parameters');

        // Optional return type: func add(int32: a, int32: b) int32 { ... } or func add(a, b): int32 { ... }
        if (this.match(':') || this.match(';')) {
            // consumed ':'
        }
        let returnType = 'void';
        if (!this.check('{') && (this.checkType() || this.check('IDENTIFIER'))) {
            returnType = this.parseType();
        }

        const body = this.block();
        return { type: 'FunctionDef', name, params, returnType, body };
    }

    varDecl(isConst) {
        let vtype = 'any';
        let name = '';
        if (this.checkType()) {
            vtype = this.parseType();
            this.consume(':', 'Expected \':\' after variable type');
            name = this.consume('IDENTIFIER', 'Expected variable name').value;
        } else {
            name = this.consume('IDENTIFIER', 'Expected variable name').value;
        }
        this.consume('=', 'Expected \'=\' in variable declaration');
        const init = this.expression();
        if (init.type !== 'StructLiteral' && init.type !== 'MapLiteral') {
            this.consume(';', "Expected ';' at the end of variable declaration", "All variable declarations in Sher must end with a semicolon ';' (e.g. var int32: i = 0;)");
        } else {
            this.match(';');
        }
        return { type: 'VarDecl', isConst, vtype, name, init };
    }

    checkType() {
        const t = this.peek().type;
        if (['INT', 'INT8', 'INT16', 'INT26', 'INT32', 'INT64', 'FLOAT', 'FLOAT8', 'FLOAT16', 'FLOAT32', 'FLOAT64', 'CHAR', 'STRING', 'STR', 'BOOL', 'BOOLEAN', 'VOID', 'ANY', 'MAP', '[', '('].includes(t)) {
            return true;
        }
        if (t === 'IDENTIFIER') {
            const nextTok = this.tokens[this.current + 1];
            if (nextTok && nextTok.type === ':') {
                return true;
            }
        }
        return false;
    }

    parseType() {
        if (this.match('[')) {
            const inner = this.parseType();
            this.consume(']', 'Expected \']\' in array type');
            return `[${inner}]`;
        }
        if (this.match('(')) {
            const types = [];
            do { types.push(this.parseType()); } while (this.match(','));
            this.consume(')', 'Expected \')\' in tuple type');
            return `(${types.join(', ')})`;
        }
        if (this.match('MAP')) {
            if (this.match('[')) {
                const k = this.parseType();
                this.consume(',', 'Expected \',\' between map key and value types');
                const v = this.parseType();
                this.consume(']', 'Expected \']\' in map type');
                return `map[${k}, ${v}]`;
            }
            return 'map[any, any]';
        }
        return this.advance().lexeme;
    }

    statement() {
        if (this.match('IF')) return this.ifStmt();
        if (this.match('WHILE')) return this.whileStmt();
        if (this.match('FOR')) return this.forStmt();
        if (this.match('PRINT')) return this.printStmt();
        if (this.match('RETURN')) {
            const val = this.check(';') ? null : this.expression();
            this.consume(';', "Expected ';' at the end of return statement", "All return statements in Sher must end with a semicolon ';' (e.g. return a + b;)");
            return { type: 'Return', value: val };
        }
        if (this.match('BREAK')) {
            this.consume(';', "Expected ';' after break", "Add a semicolon ';' after break;");
            return { type: 'Break' };
        }
        if (this.match('CONTINUE')) {
            this.consume(';', "Expected ';' after continue", "Add a semicolon ';' after continue;");
            return { type: 'Continue' };
        }
        if (this.check('{')) return { type: 'Block', stmts: this.block() };

        const expr = this.expression();
        this.consume(';', "Expected ';' at the end of statement", "All statements in Sher must end with a semicolon ';' (e.g. add(5, 5);)");
        return { type: 'ExprStmt', expr };
    }

    ifStmt() {
        this.match('(');
        const condition = this.expression();
        this.match(')');
        const thenBranch = this.block();
        let elseBranch = null;
        if (this.match('ELSE')) {
            elseBranch = this.check('IF') ? [this.declaration()] : this.block();
        }
        return { type: 'If', condition, thenBranch, elseBranch };
    }

    whileStmt() {
        this.match('(');
        const condition = this.expression();
        this.match(')');
        const body = this.block();
        return { type: 'While', condition, body };
    }

    forStmt() {
        this.match('(');
        this.match('VAR', 'LET');
        let itemType = 'any';
        if (this.checkType()) {
            itemType = this.parseType();
            this.consume(':', 'Expected \':\' after loop variable type');
        }
        const itemName = this.consume('IDENTIFIER', 'Expected loop variable name').value;
        this.consume('IN', 'Expected \'in\' in for loop');
        const iterable = this.expression();
        this.match(')');
        const body = this.block();
        return { type: 'ForIn', itemName, itemType, iterable, body };
    }

    printStmt() {
        const hasParen = this.match('(');
        const args = [];
        if (!this.check(';') && !this.check(')')) {
            do { args.push(this.expression()); } while (this.match(','));
        }
        if (hasParen) this.consume(')', 'Expected \')\' after print arguments', 'Close the print call with \')\'');
        this.consume(';', "Expected ';' at the end of print statement", "All print statements in Sher must end with a semicolon ';' (e.g. print(\"Hello\");)");
        return { type: 'Print', args };
    }

    block() {
        this.consume('{', 'Expected \'{\'');
        const stmts = [];
        while (!this.check('}') && !this.isAtEnd()) {
            stmts.push(this.declaration());
        }
        this.consume('}', 'Expected \'}\'');
        return stmts;
    }

    expression() {
        return this.assignment();
    }

    assignment() {
        let expr = this.range();

        if (this.match('=')) {
            const value = this.assignment();
            return { type: 'Assign', target: expr, value };
        }
        if (this.match('+=', '-=', '*=', '/=', '%=')) {
            const op = this.previous().type;
            const value = this.assignment();
            return { type: 'CompoundAssign', target: expr, op, value };
        }
        return expr;
    }

    range() {
        const expr = this.logicalOr();
        if (this.match('..')) {
            const end = this.logicalOr();
            return { type: 'Range', start: expr, end };
        }
        return expr;
    }

    logicalOr() {
        let expr = this.logicalAnd();
        while (this.match('||')) {
            const right = this.logicalAnd();
            expr = { type: 'Binary', left: expr, op: '||', right };
        }
        return expr;
    }

    logicalAnd() {
        let expr = this.equality();
        while (this.match('&&')) {
            const right = this.equality();
            expr = { type: 'Binary', left: expr, op: '&&', right };
        }
        return expr;
    }

    equality() {
        let expr = this.comparison();
        while (this.match('==', '!=')) {
            const op = this.previous().type;
            const right = this.comparison();
            expr = { type: 'Binary', left: expr, op, right };
        }
        return expr;
    }

    comparison() {
        let expr = this.addition();
        while (this.match('<', '<=', '>', '>=')) {
            const op = this.previous().type;
            const right = this.addition();
            expr = { type: 'Binary', left: expr, op, right };
        }
        return expr;
    }

    addition() {
        let expr = this.multiplication();
        while (this.match('+', '-')) {
            const op = this.previous().type;
            const right = this.multiplication();
            expr = { type: 'Binary', left: expr, op, right };
        }
        return expr;
    }

    multiplication() {
        let expr = this.unary();
        while (this.match('*', '/', '%')) {
            const op = this.previous().type;
            const right = this.unary();
            expr = { type: 'Binary', left: expr, op, right };
        }
        return expr;
    }

    unary() {
        if (this.match('!', '-')) {
            const op = this.previous().type;
            const right = this.unary();
            return { type: 'Unary', op, expr: right };
        }
        return this.call();
    }

    call() {
        let expr = this.primary();

        while (true) {
            if (this.match('(')) {
                const args = [];
                if (!this.check(')')) {
                    do { args.push(this.expression()); } while (this.match(','));
                }
                this.consume(')', 'Expected \')\' after arguments');
                expr = { type: 'Call', callee: expr, args };
            } else if (this.match('[')) {
                const index = this.expression();
                this.consume(']', 'Expected \']\' after index');
                expr = { type: 'Index', target: expr, index };
            } else if (this.match('.')) {
                const field = this.advance().value || this.previous().lexeme;
                expr = { type: 'FieldAccess', target: expr, field };
            } else {
                break;
            }
        }
        return expr;
    }

    primary() {
        if (this.match('TRUE')) return { type: 'Literal', value: true };
        if (this.match('FALSE')) return { type: 'Literal', value: false };
        if (this.match('NULL')) return { type: 'Literal', value: null };
        if (this.match('INT', 'FLOAT', 'STRING', 'CHAR')) {
            return { type: 'Literal', value: this.previous().value };
        }

        if (this.match('IDENTIFIER')) {
            let name = this.previous().value;
            while (this.match('::')) {
                name += '::' + this.advance().value;
            }

            if (this.check('{') && this.peekIsStructLiteral()) {
                this.advance();
                const fields = {};
                while (!this.check('}') && !this.isAtEnd()) {
                    const fname = this.consume('IDENTIFIER', 'Expected field name').value;
                    this.consume(':', 'Expected \':\' after field name');
                    fields[fname] = this.expression();
                    this.match(',', ';');
                }
                this.consume('}', 'Expected \'}\' after struct literal');
                return { type: 'StructLiteral', structName: name, fields };
            }

            return { type: 'Variable', name };
        }

        // Array literal [1, 2, 3]
        if (this.match('[')) {
            const items = [];
            if (!this.check(']')) {
                do { items.push(this.expression()); } while (this.match(','));
            }
            this.consume(']', 'Expected \']\' after array');
            return { type: 'ArrayLiteral', items };
        }

        // Map literal { "k": v } or {}
        if (this.match('{')) {
            const entries = [];
            if (!this.check('}')) {
                do {
                    const k = this.expression();
                    this.consume(':', 'Expected \':\' after map key');
                    const v = this.expression();
                    entries.push({ key: k, value: v });
                } while (this.match(',', ';'));
            }
            this.consume('}', 'Expected \'}\' after map');
            return { type: 'MapLiteral', entries };
        }

        // Tuple or grouping (x)
        if (this.match('(')) {
            if (this.check(')')) {
                this.advance();
                return { type: 'TupleLiteral', items: [] };
            }
            const first = this.expression();
            if (this.match(',')) {
                const items = [first];
                do { items.push(this.expression()); } while (this.match(','));
                this.consume(')', 'Expected \')\' after tuple');
                return { type: 'TupleLiteral', items };
            }
            this.consume(')', 'Expected \')\' after expression');
            return first;
        }

        this.error(this.peek(), `Unexpected token '${this.peek().lexeme}'`);
    }

    peekIsStructLiteral() {
        if (this.current < this.tokens.length && this.tokens[this.current].type === '{') {
            if (this.current + 2 < this.tokens.length) {
                return this.tokens[this.current + 1].type === 'IDENTIFIER' && this.tokens[this.current + 2].type === ':';
            }
        }
        return false;
    }
}

// ==================== INTERPRETER ====================
class SherInterpreter {
    constructor(ast, source, engine) {
        this.ast = ast;
        this.source = source;
        this.engine = engine;
        this.globals = new Map();
        this.structDefs = new Map();
        this.enumDefs = new Map();
    }

    execute() {
        for (const stmt of this.ast) {
            this.execStmt(stmt, this.globals);
        }

        // Auto-run main() if defined
        if (this.globals.has('main')) {
            const mainFn = this.globals.get('main');
            if (typeof mainFn === 'object' && mainFn.__is_fn && mainFn.params.length === 0) {
                this.callFunction(mainFn, [], this.globals);
            }
        }
    }

    execStmt(stmt, env) {
        if (!stmt) return;
        switch (stmt.type) {
            case 'StructDef':
                this.structDefs.set(stmt.name, stmt.fields);
                break;
            case 'EnumDef':
                const variants = {};
                stmt.variants.forEach((v, idx) => {
                    variants[v.name] = v.value ? this.evalExpr(v.value, env) : idx;
                });
                this.enumDefs.set(stmt.name, variants);
                break;
            case 'FunctionDef':
                env.set(stmt.name, {
                    __is_fn: true,
                    name: stmt.name,
                    params: stmt.params,
                    body: stmt.body,
                    closure: env
                });
                break;
            case 'VarDecl':
                const val = this.evalExpr(stmt.init, env);
                env.set(stmt.name, val);
                break;
            case 'Print':
                const evaluatedArgs = stmt.args.map(a => this.evalExpr(a, env));
                this.engine.emitPrint(evaluatedArgs);
                break;
            case 'If':
                if (this.isTruthy(this.evalExpr(stmt.condition, env))) {
                    return this.execBlock(stmt.thenBranch, env);
                } else if (stmt.elseBranch) {
                    return this.execBlock(stmt.elseBranch, env);
                }
                break;
            case 'While':
                while (this.isTruthy(this.evalExpr(stmt.condition, env))) {
                    const res = this.execBlock(stmt.body, env);
                    if (res && res.type === 'break') break;
                    if (res && res.type === 'return') return res;
                }
                break;
            case 'ForIn':
                let iterVal = null;
                if (stmt.iterable.type === 'Range') {
                    const start = this.evalExpr(stmt.iterable.start, env);
                    const end = this.evalExpr(stmt.iterable.end, env);
                    const rangeArr = [];
                    for (let i = start; i < end; i++) rangeArr.push(i);
                    iterVal = rangeArr;
                } else {
                    iterVal = this.evalExpr(stmt.iterable, env);
                }

                let items = [];
                if (Array.isArray(iterVal)) items = iterVal;
                else if (typeof iterVal === 'string') items = iterVal.split('');
                else if (iterVal && iterVal.__is_map) items = iterVal.entries.map(([k]) => k);
                else if (iterVal && iterVal.__is_tuple) items = iterVal.items;

                for (const item of items) {
                    const loopEnv = new Map(env);
                    loopEnv.set(stmt.itemName, item);
                    const res = this.execBlock(stmt.body, loopEnv);
                    if (res && res.type === 'break') break;
                    if (res && res.type === 'return') return res;
                }
                break;
            case 'Return':
                return { type: 'return', value: stmt.value ? this.evalExpr(stmt.value, env) : null };
            case 'Break':
                return { type: 'break' };
            case 'Continue':
                return { type: 'continue' };
            case 'ExprStmt':
                this.evalExpr(stmt.expr, env);
                break;
        }
    }

    execBlock(stmts, env) {
        for (const s of stmts) {
            const res = this.execStmt(s, env);
            if (res && (res.type === 'return' || res.type === 'break' || res.type === 'continue')) {
                return res;
            }
        }
    }

    evalExpr(expr, env) {
        if (!expr) return null;
        switch (expr.type) {
            case 'Literal':
                return expr.value;
            case 'Variable':
                if (expr.name.includes('::')) {
                    const [mod, item] = expr.name.split('::');
                    if (mod === 'math') {
                        if (item.toLowerCase() === 'pi') return Math.PI;
                        if (item.toLowerCase() === 'e') return Math.E;
                    }
                    if (this.enumDefs.has(mod)) {
                        const variants = this.enumDefs.get(mod);
                        if (variants.hasOwnProperty(item)) {
                            return { __is_enum: true, enum_name: mod, variant: item, value: variants[item] };
                        }
                    }
                }
                if (expr.name === 'math::pi' || expr.name === 'pi') return Math.PI;
                if (expr.name === 'math::e' || expr.name === 'e') return Math.E;
                if (env.has(expr.name)) return env.get(expr.name);
                if (this.globals.has(expr.name)) return this.globals.get(expr.name);
                throw new Error(`Undefined variable '${expr.name}'`);
            case 'ArrayLiteral':
                return expr.items.map(it => this.evalExpr(it, env));
            case 'TupleLiteral':
                return { __is_tuple: true, items: expr.items.map(it => this.evalExpr(it, env)) };
            case 'MapLiteral':
                const mapEntries = expr.entries.map(e => [this.evalExpr(e.key, env), this.evalExpr(e.value, env)]);
                return { __is_map: true, entries: mapEntries };
            case 'StructLiteral':
                const fieldVals = {};
                for (const [k, v] of Object.entries(expr.fields)) {
                    fieldVals[k] = this.evalExpr(v, env);
                }
                return { __is_struct: true, struct_name: expr.structName, fields: fieldVals };
            case 'Assign':
                const rval = this.evalExpr(expr.value, env);
                if (expr.target.type === 'Variable') {
                    env.set(expr.target.name, rval);
                } else if (expr.target.type === 'Index') {
                    const targetObj = this.evalExpr(expr.target.target, env);
                    const idx = this.evalExpr(expr.target.index, env);
                    if (Array.isArray(targetObj)) {
                        targetObj[idx] = rval;
                    } else if (targetObj && targetObj.__is_map) {
                        const existing = targetObj.entries.find(([k]) => k === idx);
                        if (existing) existing[1] = rval;
                        else targetObj.entries.push([idx, rval]);
                    }
                } else if (expr.target.type === 'FieldAccess') {
                    const targetObj = this.evalExpr(expr.target.target, env);
                    if (targetObj && targetObj.__is_struct) {
                        targetObj.fields[expr.target.field] = rval;
                    }
                }
                return rval;
            case 'CompoundAssign':
                const cval = this.evalExpr(expr.value, env);
                let cur = this.evalExpr(expr.target, env);
                let nextVal = cur;
                if (expr.op === '+=') nextVal = cur + cval;
                if (expr.op === '-=') nextVal = cur - cval;
                if (expr.op === '*=') nextVal = cur * cval;
                if (expr.op === '/=') nextVal = cur / cval;
                if (expr.op === '%=') nextVal = cur % cval;
                if (expr.target.type === 'Variable') {
                    env.set(expr.target.name, nextVal);
                } else if (expr.target.type === 'FieldAccess') {
                    const targetObj = this.evalExpr(expr.target.target, env);
                    if (targetObj && targetObj.__is_struct) targetObj.fields[expr.target.field] = nextVal;
                }
                return nextVal;
            case 'Index':
                const targetObj = this.evalExpr(expr.target, env);
                const idx = this.evalExpr(expr.index, env);
                if (Array.isArray(targetObj)) return targetObj[idx];
                if (typeof targetObj === 'string') return targetObj[idx];
                if (targetObj && targetObj.__is_tuple) return targetObj.items[idx];
                if (targetObj && targetObj.__is_map) {
                    const pair = targetObj.entries.find(([k]) => k === idx);
                    if (pair) return pair[1];
                    throw new Error(`Key '${idx}' not found in map`);
                }
                return null;
            case 'FieldAccess':
                const fTarget = this.evalExpr(expr.target, env);
                if (fTarget && fTarget.__is_struct) return fTarget.fields[expr.field];
                return null;
            case 'Binary':
                const l = this.evalExpr(expr.left, env);
                const r = this.evalExpr(expr.right, env);
                switch (expr.op) {
                    case '+': return l + r;
                    case '-': return l - r;
                    case '*': return l * r;
                    case '/': return l / r;
                    case '%': return l % r;
                    case '==':
                        if (l && l.__is_enum && r && r.__is_enum) return l.enum_name === r.enum_name && l.variant === r.variant;
                        if (l && l.__is_enum) return l.value === r;
                        if (r && r.__is_enum) return r.value === l;
                        return l === r;
                    case '!=':
                        if (l && l.__is_enum && r && r.__is_enum) return l.enum_name !== r.enum_name || l.variant !== r.variant;
                        return l !== r;
                    case '<': return l < r;
                    case '<=': return l <= r;
                    case '>': return l > r;
                    case '>=': return l >= r;
                    case '&&': return this.isTruthy(l) && this.isTruthy(r);
                    case '||': return this.isTruthy(l) || this.isTruthy(r);
                }
                return null;
            case 'Unary':
                const u = this.evalExpr(expr.expr, env);
                if (expr.op === '!') return !this.isTruthy(u);
                if (expr.op === '-') return -u;
                return u;
            case 'Call':
                let calleeName = '';
                if (expr.callee.type === 'Variable') calleeName = expr.callee.name;
                const args = expr.args.map(a => this.evalExpr(a, env));

                // Built-in functions
                if (calleeName === 'len' && args.length === 1) {
                    const arg = args[0];
                    if (Array.isArray(arg) || typeof arg === 'string') return arg.length;
                    if (arg && arg.__is_tuple) return arg.items.length;
                    if (arg && arg.__is_map) return arg.entries.length;
                    return 0;
                }

                // Math module functions
                if (calleeName.startsWith('math::') || ['sqrt', 'pow', 'abs', 'round', 'floor', 'ceil', 'min', 'max', 'clamp', 'random', 'randomFloat', 'sin', 'cos', 'tan', 'log'].includes(calleeName)) {
                    const fn = calleeName.replace('math::', '');
                    if (fn === 'sqrt') return Math.sqrt(args[0]);
                    if (fn === 'pow') return Math.pow(args[0], args[1]);
                    if (fn === 'abs') return Math.abs(args[0]);
                    if (fn === 'round') return Math.round(args[0]);
                    if (fn === 'floor') return Math.floor(args[0]);
                    if (fn === 'ceil') return Math.ceil(args[0]);
                    if (fn === 'min') return Math.min(args[0], args[1]);
                    if (fn === 'max') return Math.max(args[0], args[1]);
                    if (fn === 'clamp') return Math.min(Math.max(args[0], args[1]), args[2]);
                    if (fn === 'random') {
                        if (args.length === 2) return Math.floor(Math.random() * (args[1] - args[0] + 1)) + args[0];
                        return Math.random();
                    }
                    if (fn === 'randomFloat') return Math.random();
                    if (fn === 'sin') return Math.sin(args[0]);
                    if (fn === 'cos') return Math.cos(args[0]);
                    if (fn === 'tan') return Math.tan(args[0]);
                    if (fn === 'log') return Math.log(args[0]);
                }

                // Time module functions
                if (calleeName.startsWith('time::') || ['now', 'nowMillis', 'elapsed', 'sleep'].includes(calleeName)) {
                    const fn = calleeName.replace('time::', '');
                    if (fn === 'now') return Math.floor(Date.now() / 1000);
                    if (fn === 'nowMillis') return Date.now();
                    if (fn === 'elapsed') return Date.now() - args[0];
                    if (fn === 'sleep') return null;
                }

                // User-defined function
                const fnObj = env.get(calleeName) || this.globals.get(calleeName);
                if (fnObj && fnObj.__is_fn) {
                    return this.callFunction(fnObj, args, env);
                }

                // Method call on object: obj.method(args)
                if (expr.callee.type === 'FieldAccess') {
                    const targetVal = this.evalExpr(expr.callee.target, env);
                    const method = expr.callee.field;

                    if (targetVal && targetVal.__is_map) {
                        if (method === 'len') return targetVal.entries.length;
                        if (method === 'has' || method === 'contains') return targetVal.entries.some(([k]) => k === args[0]);
                        if (method === 'keys') return targetVal.entries.map(([k]) => k);
                        if (method === 'values') return targetVal.entries.map(([, v]) => v);
                        if (method === 'remove') {
                            const idx = targetVal.entries.findIndex(([k]) => k === args[0]);
                            if (idx !== -1) {
                                const removed = targetVal.entries.splice(idx, 1)[0];
                                return removed[1];
                            }
                            return null;
                        }
                        if (method === 'clear') { targetVal.entries = []; return null; }
                    }

                    if (typeof targetVal === 'string') {
                        if (method === 'len') return targetVal.length;
                        if (method === 'has' || method === 'contains') return targetVal.includes(args[0]);
                    }

                    if (Array.isArray(targetVal)) {
                        if (method === 'add' || method === 'push') { targetVal.push(args[0]); return null; }
                        if (method === 'remove' || method === 'pop') {
                            if (args.length === 1) return targetVal.splice(args[0], 1)[0];
                            return targetVal.pop();
                        }
                        if (method === 'len') return targetVal.length;
                        if (method === 'has' || method === 'contains') return targetVal.includes(args[0]);
                        if (method === 'clear') { targetVal.length = 0; return null; }
                    }
                }

                throw new Error(`Undefined function '${calleeName}'`);
        }
    }

    callFunction(fn, args, env) {
        if (args.length !== fn.params.length) {
            throw new Error(`Function '${fn.name}' expected ${fn.params.length} argument(s), got ${args.length}`);
        }
        const fnEnv = new Map(this.globals);
        for (const [k, v] of fn.closure.entries()) {
            fnEnv.set(k, v);
        }
        fn.params.forEach((p, idx) => {
            fnEnv.set(p.name, args[idx]);
        });
        const res = this.execBlock(fn.body, fnEnv);
        if (res && res.type === 'return') return res.value;
        return null;
    }

    isTruthy(val) {
        if (val === null || val === undefined || val === false || val === 0) return false;
        if (typeof val === 'string') return val.length > 0;
        if (Array.isArray(val)) return val.length > 0;
        return true;
    }
}

if (typeof module !== 'undefined' && module.exports) {
    module.exports = { SherEngine, SherLexer, SherParser, SherInterpreter };
}
if (typeof window !== 'undefined') {
    window.SherEngine = SherEngine;
}
