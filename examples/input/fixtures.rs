//! Sample text fixtures for the input sandbox example.

/// Sample text options for testing the text input/area.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SampleText {
    Typography,
    RtlMixed,
    TrickyText,
    MobyDick,
}

impl SampleText {
    pub const ALL: [SampleText; 4] = [
        Self::Typography,
        Self::RtlMixed,
        Self::TrickyText,
        Self::MobyDick,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Typography => "Typography",
            Self::RtlMixed => "RTL Mixed",
            Self::TrickyText => "Tricky Text",
            Self::MobyDick => "Moby Dick",
        }
    }

    pub fn content(&self) -> &'static str {
        match self {
            Self::Typography => TYPOGRAPHY_TEXT,
            Self::RtlMixed => RTL_MIXED_TEXT,
            Self::TrickyText => TRICKY_TEXT,
            Self::MobyDick => MOBY_DICK_TEXT,
        }
    }
}

pub const TYPOGRAPHY_TEXT: &str = r#"The quick brown fox jumps over the lazy dog.

ABCDEFGHIJKLMNOPQRSTUVWXYZ
abcdefghijklmnopqrstuvwxyz
0123456789

Typography is the art and technique of arranging type to make written language legible, readable and appealing when displayed.

The arrangement of type involves selecting typefaces, point sizes, line lengths, line-spacing (leading), and letter-spacing (tracking), as well as adjusting the space between pairs of letters (kerning).

"Typography is what language looks like."
— Ellen Lupton"#;

pub const RTL_MIXED_TEXT: &str = r#"English text with العربية mixed in.

שלום עולם - Hello World in Hebrew

This line has مرحبا Arabic in the middle.

Numbers work in RTL too: ٠١٢٣٤٥٦٧٨٩

Mixed: Hello مرحبا שלום World

Right-to-left scripts:
• Arabic: مرحبا بالعالم
• Hebrew: שלום עולם
• Persian: سلام دنیا"#;

pub const TRICKY_TEXT: &str = r#"Emoji sequences: 👨‍👩‍👧‍👦 👩🏽‍🚀 🏳️‍🌈 👍🏻👍🏿

Combining characters: é = e + ́  (U+0065 + U+0301)
Precomposed: é (U+00E9)

Zero-width joiners: 👨‍💻 (man + ZWJ + laptop)

Variation selectors: ☺︎ (text) vs ☺️ (emoji)

Ligatures: ff fi fl ffi ffl

Diacritics: Ṧ̈́ C̨̃ Ą̂

Zalgo text: H̴̡̧̨̢̡̧̛̛̛̙̣̫̲̼̦̫̪̦̤͎̺̱̩̦̗̼̮̱̘̭̝̗̥̙̻̺̙̫̣̦̠̯̜̮͍͓̟̘͔͚̫̮̬̥͙̼̖̝̱̼̩̙̙̭̺͕̖̪̗̖̠̋̒̿̀̅̓̓̆̒́̃͒͆̀̊̋̿̀̅̑̎̏̌̈́̏̊͗̀̋̒́̈̽̇̏̄̾̎̍̂̓͆̏͊̊̉̐͆̇̌̊̏̕͘̚̕͜͜͜͝͝͝͠͠ͅȩ̷̧̨̢̛̛̮̲͉̲̦̙̪̫̻̠̭̖̞̲̱̭͎͓̪̱̺̗͕̮̳̫͕͙̻̪̗̤̼̥̣̝̲̫̬͙̻̮̟̤̠̥̤̣̞͉̮̤̻̱̮̙̮͇̫̯̭̬̰͕̦̲̞͉͉̗͍̖̦̞̭̳̖̖͔͉̞̊́͌̀̓̿̇̈́̈́̈́̃̋̄̐̓̎̓̒̍̌̇̿̌̓͆̉̀̉̽́̔̔̃̉̆̓̋̿̅̈́͗̇̆̈́̓̌̑̽͊͛́͆͐̓͆̓̾͒͛̈͛̅̓̒͗̀̈̚̚͘̚̕͜͜͝͠͝͠͝͝͝ͅl̴̨̨̧̡̡̡̡̛̛̖̳̼̙̘̜̦̺̻͚̙̩̪͓̬̬͔̳̬͚͇̫̬̰̤̲̝̰̝̼̮̫̘͕̺͙̪̩̮͙̰̼͇͎͈̦̜̮̝̙̺̖͉̭͔̈̆̏̾̊̌̿͒̆͋́̿͗͂̓͂̎̌̒͐́͗̈̈́̐̃̓̀͐̂͛̿̉͒̓͐͂͆̓͌̇̑͒̑͂̋̎̃̈́͂̄͗̏͑͌̕͘̚̚̚̚̕̕͜͜͜͝͠ͅͅļ̵̨̢̢̧̡̛̛̛̛̙̱̣̥͔̤̬̭̙̲͍̱̪̼̭͍̯͍̯̝̬̝͈̤̼̱̳̠̲̗̯̺̝͇̙̤̩͔̫̦̞̦̱̭̟̖̙̪͓̜̫̗̪̯̳̗̙̩̠̬̠̠̫̼̰͍̯̻̤͓̦̟̬̋̾̀͌̆͆̑͛̇̾̏͛͗̿̐̃̿́̃̄̏̑͑͆̿̍̐̉̈̔̈́͂̇̇̅͛̏̓̂͆͑̈́̇̈́͑͌͑̀̐͊̌̒̿̂̆̑̓̕̕̕̕̕͘̕̚͜͜͠͝͠͠͠͝ͅǫ̴̡̡̨̢̛͇͕̰̟̞̖̼̘̙͈̺̜̱͈͇̫̱̞̠̺̳̤̯̤̟̞̗̰̲̺̫̝͙̳̺̤͓̹̘̫̼͇̫̪͖̤̮͔̖͔̟̩"#;

pub const MOBY_DICK_TEXT: &str = r#"Call me Ishmael. Some years ago—never mind how long precisely—having little or no money in my purse, and nothing particular to interest me on shore, I thought I would sail about a little and see the watery part of the world.

It is a way I have of driving off the spleen and regulating the circulation. Whenever I find myself growing grim about the mouth; whenever it is a damp, drizzly November in my soul; whenever I find myself involuntarily pausing before coffin warehouses, and bringing up the rear of every funeral I meet; and especially whenever my hypos get such an upper hand of me, that it requires a strong moral principle to prevent me from deliberately stepping into the street, and methodically knocking people's hats off—then, I account it high time to get to sea as soon as I can.

This is my substitute for pistol and ball. With a philosophical flourish Cato throws himself upon his sword; I quietly take to the ship. There is nothing surprising in this. If they but knew it, almost all men in their degree, some time or other, cherish very nearly the same feelings towards the ocean with me.

There now is your insular city of the Manhattoes, belted round by wharves as Indian isles by coral reefs—commerce surrounds it with her surf. Right and left, the streets take you waterward."#;
