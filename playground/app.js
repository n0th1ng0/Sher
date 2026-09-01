/**
 * Sher Playground + Holy Sher Documentation
 */

const INITIAL_CODE = `func main() {
    print("Hello, Sher!");
}`;

const LESSONS = {
    pl: [
        {
            name: "Rozdział 1: Wprowadzenie",
            kicker: "Rozdział 1",
            title: "Wprowadzenie do języka Sher",
            lead: "Sher jest statycznie typowanym językiem proceduralnym. Został zaprojektowany z myślą o prostocie, determinizmie oraz czytelności kodu źródłowego.",
            sections: [
                {
                    title: "Program główny",
                    desc: "Każdy samodzielny program w Sher rozpoczyna swoje działanie od funkcji main. Instrukcje wykonywane są sekwencyjnie.",
                    code: `func main() {
    print("Witaj w języku Sher!");
}`
                }
            ]
        },
        {
            name: "Rozdział 2: Zmienne i Stałe",
            kicker: "Rozdział 2",
            title: "Zmienne i Stałe",
            lead: "W języku Sher zarządzanie stanem opiera się na wyraźnym podziale na stałe wartości oraz zmienne mutowalne.",
            sections: [
                {
                    title: "Stałe niemutowalne (let)",
                    desc: "Wartości zadeklarowane za pomocą słowa 'let' są stałe i nie mogą być modyfikowane po utworzeniu.",
                    code: `func main() {
    let string: wersja = "Sher 1.0";
    print("Projekt:", wersja);
}`
                },
                {
                    title: "Zmienne modyfikowalne (var)",
                    desc: "Gdy wartość musi ulegać zmianie w czasie działania programu, deklaruje się ją za pomocą słowa 'var'.",
                    code: `func main() {
    var int32: licznik = 0;
    licznik += 1;
    print("Licznik:", licznik);
}`
                }
            ]
        },
        {
            name: "Rozdział 3: Typy Liczbowe",
            kicker: "Rozdział 3",
            title: "Typy Liczbowe",
            lead: "System typów liczbowych zapewnia precyzyjną kontrolę nad rozmiarem danych w pamięci.",
            sections: [
                {
                    title: "Liczby całkowite i zmiennoprzecinkowe",
                    desc: "Dostępne są typy całkowite (int8, int16, int32, int64) oraz zmiennoprzecinkowe (float32, float64).",
                    code: `func main() {
    let int32: wiek = 25;
    let float64: pi = 3.14159;
    print("Wiek:", wiek, "Pi:", pi);
}`
                }
            ]
        },
        {
            name: "Rozdział 4: Napisy i Znaki",
            kicker: "Rozdział 4",
            title: "Napisy i Znaki",
            lead: "Sher obsługuje ciągi tekstowe typu string oraz pojedyncze znaki char.",
            sections: [
                {
                    title: "Operacje na tekście",
                    desc: "Napisy można łączyć, sprawdzać ich długość metodą .len() oraz indeksować.",
                    code: `func main() {
    let string: imie = "Sher";
    let char: inicjal = 'S';
    print("Jezyk:", imie, "Inicjal:", inicjal);
    print("Dlugosc napisu:", imie.len());
}`
                }
            ]
        },
        {
            name: "Rozdział 5: Typ Logiczny i Warunki",
            kicker: "Rozdział 5",
            title: "Typ Logiczny i Warunki",
            lead: "Instrukcje warunkowe if/else pozwalają na rozgałęzianie logiki w oparciu o wartości typu bool.",
            sections: [
                {
                    title: "Instrukcja if oraz else",
                    desc: "Warunek w instrukcji if musi być wyrażeniem logicznym zwracającym true lub false.",
                    code: `func main() {
    var int32: punkty = 75;

    if (punkty >= 50) {
        print("Wynik pozytywny");
    } else {
        print("Wynik negatywny");
    }
}`
                }
            ]
        },
        {
            name: "Rozdział 6: Pętla While",
            kicker: "Rozdział 6",
            title: "Pętla While",
            lead: "Pętla while powtarza blok instrukcji dopóki określony warunek pozostaje spełniony.",
            sections: [
                {
                    title: "Iteracja warunkowa",
                    desc: "W ciele pętli można modyfikować zmienną kontrolną, aby zakończyć wykonywanie pętli.",
                    code: `func main() {
    var int32: i = 0;
    while (i < 3) {
        print("Krok:", i);
        i += 1;
    }
}`
                }
            ]
        },
        {
            name: "Rozdział 7: Pętla For-In",
            kicker: "Rozdział 7",
            title: "Pętla For-In",
            lead: "Pętla for-in umożliwia deterministyczną iterację po zakresach liczb oraz po ciągach znaków.",
            sections: [
                {
                    title: "Iteracja zakresowa",
                    desc: "Zakres tworzy się operatorem '..' (np. 0..4 oznacza liczby od 0 do 3 włącznie).",
                    code: `func main() {
    for (let int32: n in 0..4) {
        print("Wartosc:", n);
    }
}`
                }
            ]
        },
        {
            name: "Rozdział 8: Funkcje",
            kicker: "Rozdział 8",
            title: "Funkcje",
            lead: "Funkcje stanowią podstawową jednostkę modularyzacji kodu źródłowego.",
            sections: [
                {
                    title: "Definiowanie i wywoływanie funkcji",
                    desc: "Parametry funkcji deklaruje się w formacie 'typ: nazwa'. Typ zwracany umieszcza się przed klamrą otwierającą.",
                    code: `func dodaj(int32: a, int32: b) int32 {
    return a + b;
}

func main() {
    var int32: suma = dodaj(10, 20);
    print("Suma:", suma);
}`
                }
            ]
        },
        {
            name: "Rozdział 9: Tablice",
            kicker: "Rozdział 9",
            title: "Tablice",
            lead: "Tablice dynamiczne przechowują uporządkowane sekwencje elementów tego samego typu.",
            sections: [
                {
                    title: "Tablica dynamiczna [T]",
                    desc: "Elementy tablicy indeksowane są od zera. Metoda .add() dołącza nową wartość, a .len() zwraca liczbę elementów.",
                    code: `func main() {
    var [int32]: liczby = [10, 20];
    liczby.add(30);
    print("Liczby:", liczby);
    print("Rozmiar tablicy:", liczby.len());
}`
                }
            ]
        },
        {
            name: "Rozdział 10: Słowniki (map)",
            kicker: "Rozdział 10",
            title: "Słowniki (map)",
            lead: "Słowniki asocjacyjne przechowują pary klucz-wartość o ściśle określonych typach.",
            sections: [
                {
                    title: "Struktura map[K, V]",
                    desc: "Słowniki umożliwiają szybki odczyt i zapis pod kluczem oraz pobranie listy kluczy metodą .keys().",
                    code: `func main() {
    var map[string, int32]: punkty = {
        "Anna": 95,
        "Jan": 80
    };
    punkty["Marek"] = 88;
    print("Punkty Anny:", punkty["Anna"]);
    print("Lista graczy:", punkty.keys());
}`
                }
            ]
        },
        {
            name: "Rozdział 11: Struktury (struct)",
            kicker: "Rozdział 11",
            title: "Struktury (struct)",
            lead: "Struktury pozwalają na modelowanie złożonych obiektów domenowych poprzez grupowanie powiązanych pól.",
            sections: [
                {
                    title: "Deklaracja i inicjalizacja",
                    desc: "Struktura definiuje pola o określonych typach. Tworzenie instancji następuje przez podanie wartości pól.",
                    code: `struct Gracz {
    string: imie;
    int32: hp;
}

func main() {
    var Gracz: g = Gracz { imie: "Rycerz", hp: 100 };
    g.hp -= 20;
    print("Bohater:", g.imie, "HP:", g.hp);
}`
                }
            ]
        },
        {
            name: "Rozdział 12: Enumy (enum)",
            kicker: "Rozdział 12",
            title: "Enumy (enum)",
            lead: "Typy wyliczeniowe reprezentują skończony zestaw nazwanych wariantów stanu w programie.",
            sections: [
                {
                    title: "Warianty wyliczenia",
                    desc: "Warianty enumów mogą mieć automatyczne lub jawnie przypisane wartości liczbowe.",
                    code: `enum Status {
    Oczekuje,
    Aktywny,
    Gotowe = 200
}

func main() {
    var Status: s = Status::Aktywny;
    if (s == Status::Aktywny) {
        print("Status: w toku");
    }
}`
                }
            ]
        }
    ],

    en: [
        {
            name: "Chapter 1: Introduction",
            kicker: "Chapter 1",
            title: "Introduction to Sher",
            lead: "Sher is a statically-typed procedural programming language designed for simplicity, determinism, and architectural clarity.",
            sections: [
                {
                    title: "Main Program",
                    desc: "Every standalone Sher program begins execution inside the main function. Statements execute sequentially.",
                    code: `func main() {
    print("Welcome to Sher!");
}`
                }
            ]
        },
        {
            name: "Chapter 2: Variables and Constants",
            kicker: "Chapter 2",
            title: "Variables and Constants",
            lead: "State management in Sher relies on an explicit separation between immutable constants and mutable variables.",
            sections: [
                {
                    title: "Immutable Constants (let)",
                    desc: "Values bound using 'let' are immutable and cannot be modified once declared.",
                    code: `func main() {
    let string: version = "Sher 1.0";
    print("Project:", version);
}`
                },
                {
                    title: "Mutable Variables (var)",
                    desc: "When a value must change during runtime, declare the variable using 'var'.",
                    code: `func main() {
    var int32: counter = 0;
    counter += 1;
    print("Counter:", counter);
}`
                }
            ]
        },
        {
            name: "Chapter 3: Numeric Types",
            kicker: "Chapter 3",
            title: "Numeric Types",
            lead: "The numeric type system provides explicit control over memory widths and performance.",
            sections: [
                {
                    title: "Integers and Floats",
                    desc: "Sher provides integer types (int8, int16, int32, int64) and floating-point types (float32, float64).",
                    code: `func main() {
    let int32: age = 25;
    let float64: pi = 3.14159;
    print("Age:", age, "Pi:", pi);
}`
                }
            ]
        },
        {
            name: "Chapter 4: Strings and Chars",
            kicker: "Chapter 4",
            title: "Strings and Chars",
            lead: "Sher supports UTF-8 strings and individual character literals.",
            sections: [
                {
                    title: "String operations",
                    desc: "Strings can be indexed, inspected with .len(), and passed to functions.",
                    code: `func main() {
    let string: name = "Sher";
    let char: initial = 'S';
    print("Name:", name, "Initial:", initial);
    print("Length:", name.len());
}`
                }
            ]
        },
        {
            name: "Chapter 5: Booleans and If-Else",
            kicker: "Chapter 5",
            title: "Booleans and If-Else",
            lead: "Conditionals allow branching based on boolean expressions.",
            sections: [
                {
                    title: "If and Else statements",
                    desc: "Conditions evaluate strictly to boolean values without implicit truthiness conversions.",
                    code: `func main() {
    var int32: score = 75;

    if (score >= 50) {
        print("Passed");
    } else {
        print("Failed");
    }
}`
                }
            ]
        },
        {
            name: "Chapter 6: While Loops",
            kicker: "Chapter 6",
            title: "While Loops",
            lead: "The while loop executes its block repeatedly as long as the condition remains true.",
            sections: [
                {
                    title: "Conditional iteration",
                    desc: "The loop variable is updated within the block to ensure deterministic termination.",
                    code: `func main() {
    var int32: i = 0;
    while (i < 3) {
        print("Step:", i);
        i += 1;
    }
}`
                }
            ]
        },
        {
            name: "Chapter 7: For-In Loops",
            kicker: "Chapter 7",
            title: "For-In Loops",
            lead: "The for-in loop enables clean iteration over ranges and character sequences.",
            sections: [
                {
                    title: "Range iteration",
                    desc: "Use the '..' operator to define a half-open integer range (e.g. 0..4 covers 0 to 3).",
                    code: `func main() {
    for (let int32: n in 0..4) {
        print("Value:", n);
    }
}`
                }
            ]
        },
        {
            name: "Chapter 8: Functions",
            kicker: "Chapter 8",
            title: "Functions",
            lead: "Functions serve as the primary procedural building blocks.",
            sections: [
                {
                    title: "Declaration and return values",
                    desc: "Parameters follow the 'type: name' format. The return type is placed right before the opening brace.",
                    code: `func add(int32: a, int32: b) int32 {
    return a + b;
}

func main() {
    var int32: sum = add(10, 20);
    print("Sum:", sum);
}`
                }
            ]
        },
        {
            name: "Chapter 9: Arrays",
            kicker: "Chapter 9",
            title: "Arrays",
            lead: "Dynamic arrays store ordered sequences of identically typed elements.",
            sections: [
                {
                    title: "Dynamic array [T]",
                    desc: "Arrays are zero-indexed. Use .add() to append elements and .len() to read the length.",
                    code: `func main() {
    var [int32]: items = [10, 20];
    items.add(30);
    print("Items:", items);
    print("Length:", items.len());
}`
                }
            ]
        },
        {
            name: "Chapter 10: Maps",
            kicker: "Chapter 10",
            title: "Maps",
            lead: "Associative maps store typed key-value pairs with efficient lookups.",
            sections: [
                {
                    title: "Map structure map[K, V]",
                    desc: "Maps support bracket index access and built-in methods such as .keys() and .len().",
                    code: `func main() {
    var map[string, int32]: scores = {
        "Anna": 95,
        "John": 80
    };
    scores["Mark"] = 88;
    print("Anna score:", scores["Anna"]);
    print("Keys:", scores.keys());
}`
                }
            ]
        },
        {
            name: "Chapter 11: Structs",
            kicker: "Chapter 11",
            title: "Structs",
            lead: "Structs model complex domain entities by grouping named typed fields.",
            sections: [
                {
                    title: "Declaration and initialization",
                    desc: "Define custom record types and instantiate them with field assignments.",
                    code: `struct Player {
    string: name;
    int32: hp;
}

func main() {
    var Player: p = Player { name: "Knight", hp: 100 };
    p.hp -= 20;
    print("Player:", p.name, "HP:", p.hp);
}`
                }
            ]
        },
        {
            name: "Chapter 12: Enums",
            kicker: "Chapter 12",
            title: "Enums",
            lead: "Enumerations represent a finite set of named variant states.",
            sections: [
                {
                    title: "Enum variants",
                    desc: "Variants can carry implicit indices or explicit numeric discriminant values.",
                    code: `enum Status {
    Pending,
    Active,
    Done = 200
}

func main() {
    var Status: s = Status::Active;
    if (s == Status::Active) {
        print("Status: in progress");
    }
}`
                }
            ]
        }
    ]
};

// Playground DOM Elements
const codeInput = document.getElementById('code-input');
const highlightCode = document.getElementById('highlight-code');
const highlightLayer = document.getElementById('highlight-layer');
const lineNumbers = document.getElementById('line-numbers');
const terminal = document.getElementById('terminal');
const btnRun = document.getElementById('btn-run');
const btnLearn = document.getElementById('btn-learn');

// Book DOM Elements
const bookView = document.getElementById('holy-sher-view');
const btnCloseBook = document.getElementById('btn-close-book');
const chaptersNav = document.getElementById('chapters-nav');
const chapterMain = document.getElementById('chapter-main');
const btnLangPl = document.getElementById('btn-lang-pl');
const btnLangEn = document.getElementById('btn-lang-en');

let currentLang = 'pl';
let currentChapterIndex = 0;

// Initialize Engine
const engine = new SherEngine();
engine.setOutputCallback((type, text) => {
    appendTerminalLine(text, 'out-line');
});

// Syntax Highlighting Engine
function escapeHtml(text) {
    return text
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;');
}

function highlightSyntax(code) {
    const tokens = [];
    let i = 0;

    const keywords = /^(func|let|var|struct|enum|map|import|if|else|while|for|in|return|break|continue|print|true|false|null)\b/;
    const types = /^(int8|int16|int26|int32|int64|int|float8|float16|float32|float64|float|char|string|str|bool|boolean|void|any)\b/;
    const builtins = /^(math::\w+|time::\w+|io::\w+)/;
    const numbers = /^(0x[0-9a-fA-F]+|\d+(\.\d+)?)\b/;
    const strings = /^("(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*')/;
    const comments = /^(\/\/[^\n]*)/;
    const funcs = /^([a-zA-Z_]\w*)(?=\s*\()/;
    const identifiers = /^([a-zA-Z_]\w*)/;
    const operators = /^(\.\.|::|==|!=|<=|>=|\+=|-=|\*=|(?:\/=)|%=|&&|\|\||\+\+|--|[+\-*\/%<>=!&|^~?:])/;

    while (i < code.length) {
        const slice = code.slice(i);

        // Whitespace
        const ws = slice.match(/^[ \t\r\n]+/);
        if (ws) {
            tokens.push(escapeHtml(ws[0]));
            i += ws[0].length;
            continue;
        }

        // Comments
        const comm = slice.match(comments);
        if (comm) {
            tokens.push(`<span class="tok-comment">${escapeHtml(comm[0])}</span>`);
            i += comm[0].length;
            continue;
        }

        // Strings
        const str = slice.match(strings);
        if (str) {
            tokens.push(`<span class="tok-str">${escapeHtml(str[0])}</span>`);
            i += str[0].length;
            continue;
        }

        // Builtins
        const bi = slice.match(builtins);
        if (bi) {
            tokens.push(`<span class="tok-builtin">${escapeHtml(bi[0])}</span>`);
            i += bi[0].length;
            continue;
        }

        // Keywords
        const kw = slice.match(keywords);
        if (kw) {
            tokens.push(`<span class="tok-kw">${escapeHtml(kw[0])}</span>`);
            i += kw[0].length;
            continue;
        }

        // Types
        const ty = slice.match(types);
        if (ty) {
            tokens.push(`<span class="tok-type">${escapeHtml(ty[0])}</span>`);
            i += ty[0].length;
            continue;
        }

        // Functions
        const fn = slice.match(funcs);
        if (fn) {
            tokens.push(`<span class="tok-func">${escapeHtml(fn[0])}</span>`);
            i += fn[0].length;
            continue;
        }

        // Numbers
        const num = slice.match(numbers);
        if (num) {
            tokens.push(`<span class="tok-num">${escapeHtml(num[0])}</span>`);
            i += num[0].length;
            continue;
        }

        // Operators
        const op = slice.match(operators);
        if (op) {
            tokens.push(`<span class="tok-op">${escapeHtml(op[0])}</span>`);
            i += op[0].length;
            continue;
        }

        // Identifiers
        const id = slice.match(identifiers);
        if (id) {
            tokens.push(escapeHtml(id[0]));
            i += id[0].length;
            continue;
        }

        tokens.push(escapeHtml(code[i]));
        i++;
    }

    return tokens.join('');
}

function updateEditor() {
    const code = codeInput.value;
    highlightCode.innerHTML = highlightSyntax(code);
    
    // Line Numbers
    const lineCount = code.split('\n').length;
    let numbersHtml = '';
    for (let l = 1; l <= lineCount; l++) {
        numbersHtml += `<div class="ln">${l}</div>`;
    }
    lineNumbers.innerHTML = numbersHtml;
}

function syncScroll() {
    highlightLayer.scrollTop = codeInput.scrollTop;
    highlightLayer.scrollLeft = codeInput.scrollLeft;
    lineNumbers.scrollTop = codeInput.scrollTop;
}

function appendTerminalLine(text, className = '') {
    const el = document.createElement('div');
    el.className = className;
    el.textContent = text;
    terminal.appendChild(el);
    terminal.scrollTop = terminal.scrollHeight;
}

function runCode() {
    const code = codeInput.value.trim();
    if (!code) return;

    terminal.innerHTML = '';
    const result = engine.run(code);

    if (!result.success) {
        appendTerminalLine(result.formatted, 'out-err');
    }
}

// ==================== BOOK CONTROLLER ====================
function renderBookNavigation() {
    const list = LESSONS[currentLang] || LESSONS.pl;
    let html = '';
    list.forEach((ch, idx) => {
        const isActive = idx === currentChapterIndex ? 'active' : '';
        html += `<button class="chapter-btn ${isActive}" data-index="${idx}">${escapeHtml(ch.name)}</button>`;
    });
    chaptersNav.innerHTML = html;

    chaptersNav.querySelectorAll('.chapter-btn').forEach(btn => {
        btn.addEventListener('click', (e) => {
            currentChapterIndex = parseInt(e.currentTarget.getAttribute('data-index'), 10);
            renderBook();
            chapterMain.scrollTop = 0;
        });
    });
}

function renderBook() {
    renderBookNavigation();
    const list = LESSONS[currentLang] || LESSONS.pl;
    const ch = list[currentChapterIndex] || list[0];

    let sectionsHtml = '';
    ch.sections.forEach(sec => {
        sectionsHtml += `
            <div class="ch-section">
                <h2 class="ch-section-title">${escapeHtml(sec.title)}</h2>
                <p class="ch-section-desc">${escapeHtml(sec.desc)}</p>
                <div class="ch-code-box">
                    <pre><code>${highlightSyntax(sec.code)}</code></pre>
                </div>
            </div>
        `;
    });

    const prevLabel = currentLang === 'pl' ? 'Poprzedni' : 'Previous';
    const nextLabel = currentLang === 'pl' ? 'Następny' : 'Next';

    const prevBtn = currentChapterIndex > 0
        ? `<button id="btn-prev" class="btn-nav-book">${prevLabel}</button>`
        : `<div class="ch-nav-spacer"></div>`;

    const nextBtn = currentChapterIndex < list.length - 1
        ? `<button id="btn-next" class="btn-nav-book">${nextLabel}</button>`
        : `<div class="ch-nav-spacer"></div>`;

    chapterMain.innerHTML = `
        <div class="chapter-wrap">
            <div class="ch-kicker">${escapeHtml(ch.kicker)}</div>
            <h1 class="ch-title">${escapeHtml(ch.title)}</h1>
            <p class="ch-lead">${escapeHtml(ch.lead)}</p>
            ${sectionsHtml}
            <div class="ch-pagination">
                ${prevBtn}
                ${nextBtn}
            </div>
        </div>
    `;

    // Pagination events
    const prevEl = document.getElementById('btn-prev');
    if (prevEl) {
        prevEl.addEventListener('click', () => {
            if (currentChapterIndex > 0) {
                currentChapterIndex--;
                renderBook();
                chapterMain.scrollTop = 0;
            }
        });
    }

    const nextEl = document.getElementById('btn-next');
    if (nextEl) {
        nextEl.addEventListener('click', () => {
            if (currentChapterIndex < list.length - 1) {
                currentChapterIndex++;
                renderBook();
                chapterMain.scrollTop = 0;
            }
        });
    }
}

function openBook() {
    renderBook();
    bookView.classList.remove('hidden');
}

function closeBook() {
    bookView.classList.add('hidden');
}

// Event Listeners
codeInput.addEventListener('input', updateEditor);
codeInput.addEventListener('scroll', syncScroll);

codeInput.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
        e.preventDefault();
        runCode();
        return;
    }

    if (e.key === 'Tab') {
        e.preventDefault();
        const start = codeInput.selectionStart;
        const end = codeInput.selectionEnd;
        codeInput.value = codeInput.value.substring(0, start) + '    ' + codeInput.value.substring(end);
        codeInput.selectionStart = codeInput.selectionEnd = start + 4;
        updateEditor();
        return;
    }

    const pairs = { '{': '}', '(': ')', '[': ']', '"': '"', "'": "'" };
    if (pairs[e.key]) {
        const start = codeInput.selectionStart;
        const end = codeInput.selectionEnd;
        if (start === end) {
            const close = pairs[e.key];
            codeInput.value = codeInput.value.substring(0, start) + e.key + close + codeInput.value.substring(end);
            codeInput.selectionStart = codeInput.selectionEnd = start + 1;
            e.preventDefault();
            updateEditor();
        }
    }
});

btnRun.addEventListener('click', runCode);
btnLearn.addEventListener('click', openBook);
btnCloseBook.addEventListener('click', closeBook);

document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && !bookView.classList.contains('hidden')) {
        closeBook();
    }
});

btnLangPl.addEventListener('click', () => {
    currentLang = 'pl';
    btnLangPl.classList.add('active');
    btnLangEn.classList.remove('active');
    btnCloseBook.textContent = 'Powrót';
    renderBook();
});

btnLangEn.addEventListener('click', () => {
    currentLang = 'en';
    btnLangEn.classList.add('active');
    btnLangPl.classList.remove('active');
    btnCloseBook.textContent = 'Back';
    renderBook();
});

// Initial State
codeInput.value = INITIAL_CODE;
updateEditor();
