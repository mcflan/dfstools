
use std::io;

const KEYWORD: [&str; 145] = [
  "AND","DIV","EOR","MOD","OR","ERROR","LINE","OFF",
  "STEP","SPC","TAB(","ELSE","THEN","","OPENIN","PTR",
  "PAGE","TIME","LOMEM","HIMEM","ABS","ACS","ADVAL","ASC",
  "ASN","ATN","BGET","COS","COUNT","DEG","ERL","ERR",
  "EVAL","EXP","EXT","FALSE","FN","GET","INKEY","INSTR(",
  "INT","LEN","LN","LOG","NOT","OPENUP","OPENOUT","PI",
  "POINT(","POS","RAD","RND","SGN","SIN","SQR","TAN",
  "TO","TRUE","USR","VAL","VPOS","CHR$","GET$","INKEY$",
  "LEFT$(","MID$(","RIGHT$(","STR$","STRING$(","EOF","SUM","WHILE",
  "CASE","WHEN","OF","ENDCASE","OTHERWISE","ENDIF","ENDWHILE","PTR",
  "PAGE","TIME","LOMEM","HIMEM","SOUND","BPUT","CALL","CHAIN",
  "CLEAR","CLOSE","CLG","CLS","DATA","DEF","DIM","DRAW",
  "END","ENDPROC","ENVELOPE","FOR","GOSUB","GOTO","GCOL","IF",
  "INPUT","LET","LOCAL","MODE","MOVE","NEXT","ON","VDU",
  "PLOT","PRINT","PROC","READ","REM","REPEAT","REPORT","RESTORE",
  "RETURN","RUN","STOP","COLOUR","TRACE","UNTIL","WIDTH","OSCLI",
  "","CIRCLE","ELLIPSE","FILL","MOUSE","ORIGIN","QUIT","RECTANGLE",
  "SWAP","SYS","TINT","WAIT","INSTALL","","PRIVATE","BY","EXIT",
];

fn detok(s: &str) -> String {

    let mut out = "".to_string();

    let mut b = s.as_bytes();
    if s.len() == 0 || b == [ 0, 255, 255 ] {
        return out;
    }

    if b[0] as usize == b.len() && *b.last().unwrap() == 13u8 {
        let ch = u16::from_le_bytes([b[1], b[2]]); 
        out = ch.to_string();
        b = &b[3..];
        if b.len() == 0 {
            return out;
        }
        if ch > 0 {
            out += " ";
        } else {
            out = "".to_string();
        }
    }
    let mut flag = 0;
    for i in 0..s.len() {
        if b[i] == 34 {
            flag ^= 1;
        }
        if flag & 1 > 0 {
            out.push(char::from(b[i]));
        } else {
            let ch: u8 = b[i];
            if ch >= 17 && ch < 128 {
                out.push(char::from(ch));
            } else if ch == 0x8D {
                let ch = b[i+1];
                let lo = ((ch << 2) & 0xc0) ^ b[i+2];
                let hi = ((ch << 4) & 0xc0) ^ b[i+3];
                let x = u16::from_le_bytes([lo, hi]);
                out += &x.to_string();
                // TODO: skip 3 from i
            } else {
                out += KEYWORD[(ch ^ 0x80) as usize];
            }
        }
    }
    return out
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    for line in stdin.lines() {
        println!("{:}", detok(&line.unwrap()));
    }
    Ok(())
}
