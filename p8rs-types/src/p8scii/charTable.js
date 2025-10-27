const CHARACTERS = [
  ["␀",  "\\0",  "U+0000", "Terminate printing", true, false, true],
  ["¹",  "\\*",  "U+00B9", "Repeat next character", false, false, false],
  ["²",  "\\#"  ,  "U+00B2", "Draw solid background", false, false, false],
  ["³",  "\\-",  "U+00B3", "Move cursor horizontally", false, false, false],
  ["⁴",  "\\|",  "U+2074", "Move cursor vertically", false, false, false],
  ["⁵",  "\\+",  "U+2075", "Move cursor", false, false, false],
  ["⁶",  "\\^",  "U+2076", "Special command", false, false, false],
  ["⁷",  "\\a",  "U+2077", "Audio command", false, false, false],
  ["⁸",  "\\b",  "U+2078", "Backspace", false, false, false],
  ["␉",  "\\t",  "U+0009", "Tab", true, false, true],
  ["␊",  "\\n",  "U+000A", "Newline", true, false, true],
  ["ᵇ",  "\\v",  "U+1D47", "Decorate previous character", false, false, false],
  ["ᶜ",  "\\f",  "U+1D9C", "Set foreground color", false, false, false],
  ["␍",  "\\r",  "U+000D", "Carriage return", true, false, true],
  ["ᵉ",  "\\14", "U+1D49", "Switch font defined at 0x5600", false, false, false],
  ["ᶠ",  "\\15", "U+1DA0", "Switch font to default", false, false, false],
  ["▮",  "",     "U+25AE", "Vertical rectangle", false, false, false],
  ["■",  "",     "U+25A0", "Filled square", false, false, false],
  ["□",  "",     "U+25A1", "Hollow square", false, false, false],
  ["⁙",  "",     "U+2059", "Five dot", false, false, false],
  ["⁘",  "",     "U+2058", "Four dot", false, false, false],
  ["‖",  "",     "U+2016", "Pause", false, false, false],
  ["◀",  "",     "U+25C0", "Back", false, false, false],
  ["▶",  "",     "U+25B6", "Forward", false, false, false],
  ["「",  "",     "U+300C", "Japanese starting quote", false, false, false],
  ["」",  "",     "U+300D", "Japanese ending quote", false, false, false],
  ["¥",  "",     "U+00A5", "Yen sign", false, false, false],
  ["•",  "",     "U+2022", "Interpunct", false, false, false],
  ["、",  "",     "U+3001", "Japanese comma", false, false, false],
  ["。",  "",     "U+3002", "Japanese full stop", false, false, false],
  ["゛",  "",     "U+309B", "Japanese dakuten", false, false, false],
  ["゜",  "",     "U+309C", "Japanese handakuten", false, false, false],
  ["␠",  "",     "U+0020", "Space", true, false, true, " "],
  ["!",  "",     "U+0021", "!", false, false, true],
  ["\"", "\\\"", "U+0022", "Double quote", false, false, true],
  ["#",  "",     "U+0023", "Number sign", false, false, true],
  ["$",  "",     "U+0024", "Dollar sign", false, false, true],
  ["%",  "",     "U+0025", "Percent sign", false, false, true],
  ["&",  "",     "U+0026", "Ampersand", false, false, true],
  ["\'", "\\\'", "U+0027", "Single quote", false, false, true],
  ["(",  "",     "U+0028", "(", false, false, true],
  [")",  "",     "U+0029", ")", false, false, true],
  ["*",  "",     "U+002A", "*", false, false, true],
  ["+",  "",     "U+002B", "+", false, false, true],
  [",",  "",     "U+002C", ",", false, false, true],
  ["-",  "",     "U+002D", "-", false, false, true],
  [".",  "",     "U+002E", ".", false, false, true],
  ["/",  "",     "U+002F", "/", false, false, true],
  ["0",  "",     "U+0030", "0", false, false, true],
  ["1",  "",     "U+0031", "1", false, false, true],
  ["2",  "",     "U+0032", "2", false, false, true],
  ["3",  "",     "U+0033", "3", false, false, true],
  ["4",  "",     "U+0034", "4", false, false, true],
  ["5",  "",     "U+0035", "5", false, false, true],
  ["6",  "",     "U+0036", "6", false, false, true],
  ["7",  "",     "U+0037", "7", false, false, true],
  ["8",  "",     "U+0038", "8", false, false, true],
  ["9",  "",     "U+0039", "9", false, false, true],
  [":",  "",     "U+003A", ":", false, false, true],
  [";",  "",     "U+003B", ";", false, false, true],
  ["<",  "",     "U+003C", "<", false, false, true],
  ["=",  "",     "U+003D", "=", false, false, true],
  [">",  "",     "U+003E", ">", false, false, true],
  ["?",  "",     "U+003F", "?", false, false, true],
  ["@",  "",     "U+0040", "@", false, false, true],
  ["A",  "",     "U+0041", "A", false, false, true],
  ["B",  "",     "U+0042", "B", false, false, true],
  ["C",  "",     "U+0043", "C", false, false, true],
  ["D",  "",     "U+0044", "D", false, false, true],
  ["E",  "",     "U+0045", "E", false, false, true],
  ["F",  "",     "U+0046", "F", false, false, true],
  ["G",  "",     "U+0047", "G", false, false, true],
  ["H",  "",     "U+0048", "H", false, false, true],
  ["I",  "",     "U+0049", "I", false, false, true],
  ["J",  "",     "U+004A", "J", false, false, true],
  ["K",  "",     "U+004B", "K", false, false, true],
  ["L",  "",     "U+004C", "L", false, false, true],
  ["M",  "",     "U+004D", "M", false, false, true],
  ["N",  "",     "U+004E", "N", false, false, true],
  ["O",  "",     "U+004F", "O", false, false, true],
  ["P",  "",     "U+0050", "P", false, false, true],
  ["Q",  "",     "U+0051", "Q", false, false, true],
  ["R",  "",     "U+0052", "R", false, false, true],
  ["S",  "",     "U+0053", "S", false, false, true],
  ["T",  "",     "U+0054", "T", false, false, true],
  ["U",  "",     "U+0055", "U", false, false, true],
  ["V",  "",     "U+0056", "V", false, false, true],
  ["W",  "",     "U+0057", "W", false, false, true],
  ["X",  "",     "U+0058", "X", false, false, true],
  ["Y",  "",     "U+0059", "Y", false, false, true],
  ["Z",  "",     "U+005A", "Z", false, false, true],
  ["[",  "",     "U+005B", "[", false, false, true],
  ["\\", "\\\\", "U+005C", "\\", false, false, true],
  ["]",  "",     "U+005D", "]", false, false, true],
  ["^",  "",     "U+005E", "Caret", false, false, true],
  ["_",  "",     "U+005F", "Underscore", false, false, true],
  ["`",  "",     "U+0060", "Backtick", false, false, true],
  ["a",  "",     "U+0061", "a", false, false, true],
  ["b",  "",     "U+0062", "b", false, false, true],
  ["c",  "",     "U+0063", "c", false, false, true],
  ["d",  "",     "U+0064", "d", false, false, true],
  ["e",  "",     "U+0065", "e", false, false, true],
  ["f",  "",     "U+0066", "f", false, false, true],
  ["g",  "",     "U+0067", "g", false, false, true],
  ["h",  "",     "U+0068", "h", false, false, true],
  ["i",  "",     "U+0069", "i", false, false, true],
  ["j",  "",     "U+006A", "j", false, false, true],
  ["k",  "",     "U+006B", "k", false, false, true],
  ["l",  "",     "U+006C", "l", false, false, true],
  ["m",  "",     "U+006D", "m", false, false, true],
  ["n",  "",     "U+006E", "n", false, false, true],
  ["o",  "",     "U+006F", "o", false, false, true],
  ["p",  "",     "U+0070", "p", false, false, true],
  ["q",  "",     "U+0071", "q", false, false, true],
  ["r",  "",     "U+0072", "r", false, false, true],
  ["s",  "",     "U+0073", "s", false, false, true],
  ["t",  "",     "U+0074", "t", false, false, true],
  ["u",  "",     "U+0075", "u", false, false, true],
  ["v",  "",     "U+0076", "v", false, false, true],
  ["w",  "",     "U+0077", "w", false, false, true],
  ["x",  "",     "U+0078", "x", false, false, true],
  ["y",  "",     "U+0079", "y", false, false, true],
  ["z",  "",     "U+007A", "z", false, false, true],
  ["{",  "",     "U+007B", "{", false, false, true],
  ["|",  "",     "U+007C", "Vertical bar", false, false, true],
  ["}",  "",     "U+007D", "}", false, false, true],
  ["~",  "",     "U+007E", "Tilde", false, false, true],
  ["○",  "",     "U+25CB", "Hollow circle", false, false, false],
  ["█",  "",     "U+2588", "Rectangle", false, false, false],
  ["▒",  "",     "U+2592", "Checkerboard", false, false, false],
  ["🐱",  "",     "U+1F431", "Jelpi", false, false, false],
  ["⬇️", "",     "U+2B07", "Down key", false, true, false],
  ["░",  "",     "U+2591", "Dot pattern", false, false, false],
  ["✽",  "",     "U+273D", "Throwing star", false, false, false],
  ["●",  "",     "U+25CF", "Ball", false, false, false],
  ["♥",  "",     "U+2665", "Heart", false, false, false],
  ["☉",  "",     "U+2609", "Eye", false, false, false],
  ["웃",  "",     "U+C6C3", "Man", false, false, false],
  ["⌂",  "",     "U+2302", "House", false, false, false],
  ["⬅️", "",     "U+2B05", "Left key", false, true, false],
  ["😐",  "",     "U+1F610", "Face", false, false, false],
  ["♪",  "",     "U+266A", "Musical note", false, false, false],
  ["🅾️", "",     "U+1F17E", "O key", false, true, false],
  ["◆",  "",     "U+25C6", "Diamond", false, false, false],
  ["…",  "",     "U+2026", "Ellipsis", false, false, false],
  ["➡️", "",     "U+27A1", "Right key", false, true, false],
  ["★",  "",     "U+2605", "Five-pointed star", false, false, false],
  ["⧗",  "",     "U+29D7", "Hourglass", false, false, false],
  ["⬆️", "",     "U+2B06", "Up key", false, true, false],
  ["ˇ",  "",     "U+02C7", "Birds", false, false, false],
  ["∧",  "",     "U+2227", "Sawtooth", false, false, false],
  ["❎",  "",     "U+274E", "X key", false, false, false],
  ["▤",  "",     "U+25A4", "Horiz lines", false, false, false],
  ["▥",  "",     "U+25A5", "Vert lines", false, false, false],
  ["あ",  "",     "U+3042", "Hiragana a", false, false, false],
  ["い",  "",     "U+3044", "Hiragana i", false, false, false],
  ["う",  "",     "U+3046", "Hiragana u", false, false, false],
  ["え",  "",     "U+3048", "Hiragana e", false, false, false],
  ["お",  "",     "U+304A", "Hiragana o", false, false, false],
  ["か",  "",     "U+304B", "Hiragana ka", false, false, false],
  ["き",  "",     "U+304D", "Hiragana ki", false, false, false],
  ["く",  "",     "U+304F", "Hiragana ku", false, false, false],
  ["け",  "",     "U+3051", "Hiragana ke", false, false, false],
  ["こ",  "",     "U+3053", "Hiragana ko", false, false, false],
  ["さ",  "",     "U+3055", "Hiragana sa", false, false, false],
  ["し",  "",     "U+3057", "Hiragana shi", false, false, false],
  ["す",  "",     "U+3059", "Hiragana su", false, false, false],
  ["せ",  "",     "U+305B", "Hiragana se", false, false, false],
  ["そ",  "",     "U+305D", "Hiragana so", false, false, false],
  ["た",  "",     "U+305F", "Hiragana ta", false, false, false],
  ["ち",  "",     "U+3061", "Hiragana chi", false, false, false],
  ["つ",  "",     "U+3064", "Hiragana tsu", false, false, false],
  ["て",  "",     "U+3066", "Hiragana te", false, false, false],
  ["と",  "",     "U+3068", "Hiragana to", false, false, false],
  ["な",  "",     "U+306A", "Hiragana na", false, false, false],
  ["に",  "",     "U+306B", "Hiragana ni", false, false, false],
  ["ぬ",  "",     "U+306C", "Hiragana nu", false, false, false],
  ["ね",  "",     "U+306D", "Hiragana ne", false, false, false],
  ["の",  "",     "U+306E", "Hiragana no", false, false, false],
  ["は",  "",     "U+306F", "Hiragana ha", false, false, false],
  ["ひ",  "",     "U+3072", "Hiragana hi", false, false, false],
  ["ふ",  "",     "U+3075", "Hiragana fu", false, false, false],
  ["へ",  "",     "U+3078", "Hiragana he", false, false, false],
  ["ほ",  "",     "U+307B", "Hiragana ho", false, false, false],
  ["ま",  "",     "U+307E", "Hiragana ma", false, false, false],
  ["み",  "",     "U+307F", "Hiragana mi", false, false, false],
  ["む",  "",     "U+3080", "Hiragana mu", false, false, false],
  ["め",  "",     "U+3081", "Hiragana me", false, false, false],
  ["も",  "",     "U+3082", "Hiragana mo", false, false, false],
  ["や",  "",     "U+3084", "Hiragana ya", false, false, false],
  ["ゆ",  "",     "U+3086", "Hiragana yu", false, false, false],
  ["よ",  "",     "U+3088", "Hiragana yo", false, false, false],
  ["ら",  "",     "U+3089", "Hiragana ra", false, false, false],
  ["り",  "",     "U+308A", "Hiragana ri", false, false, false],
  ["る",  "",     "U+308B", "Hiragana ru", false, false, false],
  ["れ",  "",     "U+308C", "Hiragana re", false, false, false],
  ["ろ",  "",     "U+308D", "Hiragana ro", false, false, false],
  ["わ",  "",     "U+308F", "Hiragana wa", false, false, false],
  ["を",  "",     "U+3092", "Hiragana wo", false, false, false],
  ["ん",  "",     "U+3093", "Hiragana nn", false, false, false],
  ["っ",  "",     "U+3063", "Small hiragana tsu", false, false, false],
  ["ゃ",  "",     "U+3083", "Small hiragana ya", false, false, false],
  ["ゅ",  "",     "U+3085", "Small hiragana yu", false, false, false],
  ["ょ",  "",     "U+3087", "Small hiragana yo", false, false, false],
  ["ア",  "",     "U+30A2", "Katakana a", false, false, false],
  ["イ",  "",     "U+30A4", "Katakana i", false, false, false],
  ["ウ",  "",     "U+30A6", "Katakana u", false, false, false],
  ["エ",  "",     "U+30A8", "Katakana e", false, false, false],
  ["オ",  "",     "U+30AA", "Katakana o", false, false, false],
  ["カ",  "",     "U+30AB", "Katakana ka", false, false, false],
  ["キ",  "",     "U+30AD", "Katakana ki", false, false, false],
  ["ク",  "",     "U+30AF", "Katakana ku", false, false, false],
  ["ケ",  "",     "U+30B1", "Katakana ke", false, false, false],
  ["コ",  "",     "U+30B3", "Katakana ko", false, false, false],
  ["サ",  "",     "U+30B5", "Katakana sa", false, false, false],
  ["シ",  "",     "U+30B7", "Katakana shi", false, false, false],
  ["ス",  "",     "U+30B9", "Katakana su", false, false, false],
  ["セ",  "",     "U+30BB", "Katakana se", false, false, false],
  ["ソ",  "",     "U+30BD", "Katakana so", false, false, false],
  ["タ",  "",     "U+30BF", "Katakana ta", false, false, false],
  ["チ",  "",     "U+30C1", "Katakana chi", false, false, false],
  ["ツ",  "",     "U+30C4", "Katakana tsu", false, false, false],
  ["テ",  "",     "U+30C6", "Katakana te", false, false, false],
  ["ト",  "",     "U+30C8", "Katakana to", false, false, false],
  ["ナ",  "",     "U+30CA", "Katakana na", false, false, false],
  ["ニ",  "",     "U+30CB", "Katakana ni", false, false, false],
  ["ヌ",  "",     "U+30CC", "Katakana nu", false, false, false],
  ["ネ",  "",     "U+30CD", "Katakana ne", false, false, false],
  ["ノ",  "",     "U+30CE", "Katakana no", false, false, false],
  ["ハ",  "",     "U+30CF", "Katakana ha", false, false, false],
  ["ヒ",  "",     "U+30D2", "Katakana hi", false, false, false],
  ["フ",  "",     "U+30D5", "Katakana fu", false, false, false],
  ["ヘ",  "",     "U+30D8", "Katakana he", false, false, false],
  ["ホ",  "",     "U+30DB", "Katakana ho", false, false, false],
  ["マ",  "",     "U+30DE", "Katakana ma", false, false, false],
  ["ミ",  "",     "U+30DF", "Katakana mi", false, false, false],
  ["ム",  "",     "U+30E0", "Katakana mu", false, false, false],
  ["メ",  "",     "U+30E1", "Katakana me", false, false, false],
  ["モ",  "",     "U+30E2", "Katakana mo", false, false, false],
  ["ヤ",  "",     "U+30E4", "Katakana ya", false, false, false],
  ["ユ",  "",     "U+30E6", "Katakana yu", false, false, false],
  ["ヨ",  "",     "U+30E8", "Katakana yo", false, false, false],
  ["ラ",  "",     "U+30E9", "Katakana ra", false, false, false],
  ["リ",  "",     "U+30EA", "Katakana ri", false, false, false],
  ["ル",  "",     "U+30EB", "Katakana ru", false, false, false],
  ["レ",  "",     "U+30EC", "Katakana re", false, false, false],
  ["ロ",  "",     "U+30ED", "Katakana ro", false, false, false],
  ["ワ",  "",     "U+30EF", "Katakana wa", false, false, false],
  ["ヲ",  "",     "U+30F2", "Katakana wo", false, false, false],
  ["ン",  "",     "U+30F3", "Katakana n", false, false, false],
  ["ッ",  "",     "U+30C3", "Small katakana tsu", false, false, false],
  ["ャ",  "",     "U+30E3", "Small katakana ya", false, false, false],
  ["ュ",  "",     "U+30E5", "Small katakana yu", false, false, false],
  ["ョ",  "",     "U+30E7", "Small katakana yo", false, false, false],
  ["◜",  "",     "U+25DC", "Left arc", false, false, false],
  ["◝",  "",     "U+25DD", "Right arc", false, false, false],
];

let selectedChar = null;
let selectedMode = "char";

let currentTable = null;
let currentInfoBox = null;

Promise.all([createTable(), createInfoBox()]).then(([table, infoBox]) => {
  currentTable = table;
  currentInfoBox = infoBox;
  
  document.currentScript.replaceWith(
    createStyle(),
    createButtons(),
    createTableInfoWrap(currentTable, currentInfoBox),
    createNotes(),
  );
})

async function createTable() {
  if(selectedMode === "render") await loadFont();
  
  const table = document.createElement("table");
  table.classList.add("p8sciTable");
  table.classList.add(selectedMode);
  const tbody = table.createTBody();
  const headerRow = tbody.insertRow();
  headerRow.insertCell().innerHTML = "##";
  for(let c = 0; c < 16; c++) {
    headerRow.insertCell().innerHTML = `<b>${c.toString(16).toUpperCase()}</b>`;
  }
  
  for(let r = 0; r < 16; r++) {
    const row = tbody.insertRow();
    row.insertCell().innerHTML = `<b>${r.toString(16).toUpperCase()}</b>x`;
    
    for(let c = 0; c < 16; c++) {
      const charId = r * 16 + c;
      const [char, escaped, unicode, name, ctrlPic, varsel, ascii, ctrlPicChar] = CHARACTERS[charId];
      
      const cell = row.insertCell();
      if(selectedMode === "char") cell.innerHTML = char;
      else if(selectedMode === "unicode") cell.innerHTML = unicode.slice(2);
      else if(selectedMode === "render") cell.appendChild(renderChar(charId));
      
      if(ctrlPic && selectedMode === "char") cell.classList.add("dagger");
      if(varsel && selectedMode !== "render") cell.classList.add("ddagger");
      if(charId === selectedChar) cell.classList.add("selected");
      
      cell.addEventListener("click", () => {
        selectedChar = charId;
        updateTable();
        updateInfoBox();
      });
    }
  }
  
  return table;
}

async function updateTable() {
  const table = await createTable();
  currentTable.replaceWith(table);
  currentTable = table;
}

function createNotes() {
  const notes = document.createElement("p");
  notes.innerHTML = `
    <div><sup><b>†</b></sup> invisible character replaced with <a href="https://en.wikipedia.org/wiki/Control_Pictures">Control Picture</a>.</div>
    <div><sup><b>‡</b></sup> character is followed by Variation Selector-16 (U+FE0F) when converted from P8SCII to utf-8.</div>
  `;
  return notes;
}

async function createInfoBox() {
  const infoBox = document.createElement("div");
  infoBox.classList.add("tableInfo");
  
  if(typeof selectedChar === "number") {
    await loadFont();
    
    const [char, escaped, unicode, name, ctrlPic, varsel, ascii, ctrlPicChar] = CHARACTERS[selectedChar];
    
    infoBox.innerHTML = `
      ${!ctrlPic || ctrlPicChar ? `<div>Character: <code>${ctrlPicChar ?? char}</code></div>` : ""}
      <div>Name: ${name}</div>
      <div>P8SCII code: <code>${selectedChar} (0x${selectedChar.toString(16).toUpperCase()})</code></div>
      <div>Unicode: <code>${unicode}${varsel ? " U+FE0F" : ""}</code></div>
      ${escaped ? `<div>Escaped: <code>"${escaped}"</code></div>` : ""}
      <div>ASCII compatible: ${ascii ? "Yes" : "No"}</div>
    `;
    
    if(!ctrlPic || ctrlPicChar) {
    infoBox.appendChild(renderChar(selectedChar));
    }
  }
  
  return infoBox;
}

async function updateInfoBox() {
  const infoBox = await createInfoBox();
  currentInfoBox.replaceWith(infoBox);
  currentInfoBox = infoBox;
}

function createTableInfoWrap(table, box) {
  const div = document.createElement("div");
  div.classList.add("p8sciWrap");
  div.appendChild(table);
  div.appendChild(box);
  return div;
}

function createButtons() {
  const buttons = [];
  const div = document.createElement("div");
  div.classList.add("p8sciButtons");
  
  const addButton = (name, mode) => {
    const button = document.createElement("button");
    button.innerText = name;
    button.addEventListener("click", () => {
      for(const button of buttons) button.classList.remove("selected");
      button.classList.add("selected");
      
      selectedMode = mode;
      updateTable();
    });
    if(selectedMode === mode) button.classList.add("selected");
    div.appendChild(button);
    buttons.push(button);
  };
  
  addButton("Characters", "char");
  addButton("Unicode", "unicode");
  addButton("Rendered", "render");
  
  return div;
}

let fontImage = null;
let fontPromise = null;
async function loadFont() {
  if(fontImage) return;
  if(fontPromise) return await fontPromise;
  
  const image = new Image();
  fontPromise = new Promise((res, rej) => {
    image.addEventListener("load", res);
    image.addEventListener("error", rej);
  });
  image.src = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAIAAAACAAQMAAAD58POIAAAABlBMVEUAAAD///+l2Z/dAAADhklEQVRIx6VUMYgbVxAdxC9+oWKKI2whzCAWI1Jt4eIXSxiOJSxGmCOoOMJhPodIEa5YjIslDOJjrlAZXKUKn5DCXBWCSXGEsIVR6SIE178wLg+XIQSR2ZXPxE63mV0kNPv+zJv3ZgWp61L0IfmUOr0SRGafOFHSCLHrIOkz6vSm2GMC/CfSRwEj4nCIghLQ78h90Rij7wL1j2LUhGaVQiBmzXyICN2/akBi4A7GBceYYsfMcSjWJ1JHXXjfVu/o2YO2TdqSDwqECD2CbxF6CvoaATiM0kO9iCkMahL0sjBRVBZRDeLEpInetN4o0C7kNRFIMXoNiIMv1N9aQG8eJcfHCRXcKzevTsSQOm0bu35PgvLqgjqiZGNQpjHp/EEdAsXqIU2nLnqFw/+LLnhl5bnzvf9qV2CK/YrEoOapLKSJQcKO1AbVKhwSwauxntVeddTDqD1lHf2D8dWQftp+/DiMr30HPVKvRz8+HBAx3uqhW6zOHfRII/VIvfuH8f2w67fjx8F+0tdDxzzowTp1Ul+6OOgRVR9mXYf3eqR3NUbE/uqJbMldOOdkPxOE/Wq/m5eP5cLJm2vcOUVc77ZO5Eb2uz2+EkVc38xb0TOrN0+aASGytQ7d4kz2jdYYE4Ii+rm/arYFPqoAXjp3E7ev4Iou25b+givtvru/utlftZcGpVaEuJ2ZK+L5jyb/QRNSvauxZne6PhlDo7VVi4x1yW9FcPkQsHhoSpa/beKJnaVjKLkCq4jNspkg1QzbomKuCvXMGuQ7J+BtXRYqx2zpMltmD8bw+Pa0rF8U8Jv4/KRdnBHI6y/OPofi+zJ/VJJ87eD36mldYb3FDX3Z5vcJfnn59KxaXf7ars/P5zht4avl+an7eXL92JbTGaIfwwMrITltaLBoigJcZRfZ67sMJ0RqA4Ks83t55Xzz59ti0VQIxjxYH784+sPWzzyhQXDZ9O60vvMd0k8NYyZjeFhCHTv3QpW4lSPYiOSfmY0xkle40v/YLMeqNvna5ua5GHGwqMRaY2xGdooWc6iXiCi2WM+KggqiUXroohrLUqCUOGnVF2fE5K2pxQAdkSjiaGpyNpe4kNy2uSbsN3bOdrJCOP5E25Lcw+bYFbCy7caXo3i4VvIFi8jgKmoPC5qoKmOeETiyYHuE/i6NbYtst4NejE/ryhqLC7RWqbeZXTZSZI2n1uGY9/YffBjlErNyY9UAAAAASUVORK5CYII=";
  await fontPromise;
  
  fontPromise = null;
  fontImage = image;
}

function renderChar(charId) {
  if(!fontImage) throw new Error("Font not loaded");
  const x = Math.floor(charId % 16);
  const y = Math.floor(charId / 16);
  
  const canvas = document.createElement('canvas');
  canvas.width = charId > 127 ? 8 : 4;
  canvas.height = 6;
  
  const ctx = canvas.getContext("2d");
  ctx.drawImage(fontImage, x * 8, y * 8, canvas.width, canvas.height, 0, 0, canvas.width, canvas.height);
  
  return canvas;
}

function createStyle() {
  const style = document.createElement("style");
  style.appendChild(document.createTextNode(`
    .p8sciTable tr {
      background: none !important;
    }
    
    .p8sciTable td {
      position: relative;
      text-align: center;
      width: 40px;
      height: 40px;
      padding: 0 !important;
    }
    
    .p8sciTable tr:not(:first-child) td:not(:first-child) {
      background: var(--table-alt-row-background-color);
      transition: background 0.1s;
      cursor: pointer;
    }
    
    .p8sciTable.unicode tr:not(:first-child) td:not(:first-child) {
      font-family: var(--font-family-code);
      font-size: 75%;
    }
    
    .p8sciTable tr:not(:first-child) td:not(:first-child):hover {
      background: transparent;
    }
    
    .p8sciTable tr:not(:first-child) td:not(:first-child).selected {
      background: var(--sidebar-background-color);
    }
    
    .p8sciTable td::after {
      position: absolute;
      top: 2px;
      right: 2px;
      font-size: 75%;
      font-weight: bold;
    }
    
    .p8sciTable td.dagger::after {
      content: "†";
    }
    
    .p8sciTable td.ddagger::after {
      content: "‡";
    }
    
    .p8sciWrap {
      display: flex;
      flex-wrap: wrap;
      gap: 1rem;
    }
    
    .p8sciWrap .tableInfo {
      flex: 1 0 160px;
    }
    
    .p8sciWrap .tableInfo > div {
      margin-bottom: 0.5em;
      line-height: 1.2;
    }
    
    .p8sciWrap canvas {
      image-rendering: optimizeSpeed;
      image-rendering: -moz-crisp-edges;
      image-rendering: -o-crisp-edges;
      image-rendering: -webkit-optimize-contrast;
      image-rendering: pixelated;
      image-rendering: optimize-contrast;
      -ms-interpolation-mode: nearest-neighbor;
    }
    
    .p8sciTable canvas {
      height: 20px;
      border: 2px solid black;
    }
    
    .p8sciWrap .tableInfo canvas {
      height: 128px;
      width: auto;
      padding: 1px;
      background: gray;
      border: 15px solid black;
    }
    
    .p8sciButtons {
      display: flex;
      margin-bottom: 0.75em;
    }
    
    .p8sciButtons button {
      background: transparent;
      border: 1px solid var(--border-color);
      color: inherit;
      padding: 0.5em;
      transition: background 0.1s;
    }
    
    .p8sciButtons button:not(:first-child) {
      border-left: none;
    }
    
    .p8sciButtons button:first-child {
      border-radius: 0.2em 0 0 0.2em;
    }
    
    .p8sciButtons button:last-child {
      border-radius: 0 0.2em 0.2em 0;
    }
    
    .p8sciButtons button.selected {
      background: var(--sidebar-background-color);
    }
    
    .p8sciButtons button:hover {
      background: var(--sidebar-background-color);
    }
  `));
  
  return style;
}
