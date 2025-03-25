        DEF FNdetokenise(S$)
        IF S$ = "" OR S$ = CHR$0+CHR$255+CHR$255 THEN =""
        PRIVATE Keywd$()
        IF !^Keywd$()=0 THEN
          DIM Keywd$(160)
          Keywd$() = "AND","DIV","EOR","MOD","OR","ERROR","LINE","OFF",\
          \"STEP","SPC","TAB(","ELSE","THEN","","OPENIN","PTR",\
          \"PAGE","TIME","LOMEM","HIMEM","ABS","ACS","ADVAL","ASC",\
          \"ASN","ATN","BGET","COS","COUNT","DEG","ERL","ERR",\
          \"EVAL","EXP","EXT","FALSE","FN","GET","INKEY","INSTR(",\
          \"INT","LEN","LN","LOG","NOT","OPENUP","OPENOUT","PI",\
          \"POINT(","POS","RAD","RND","SGN","SIN","SQR","TAN",\
          \"TO","TRUE","USR","VAL","VPOS","CHR$","GET$","INKEY$",\
          \"LEFT$(","MID$(","RIGHT$(","STR$","STRING$(","EOF","SUM","WHILE",\
          \"CASE","WHEN","OF","ENDCASE","OTHERWISE","ENDIF","ENDWHILE","PTR",\
          \"PAGE","TIME","LOMEM","HIMEM","SOUND","BPUT","CALL","CHAIN",\
          \"CLEAR","CLOSE","CLG","CLS","DATA","DEF","DIM","DRAW",\
          \"END","ENDPROC","ENVELOPE","FOR","GOSUB","GOTO","GCOL","IF",\
          \"INPUT","LET","LOCAL","MODE","MOVE","NEXT","ON","VDU",\
          \"PLOT","PRINT","PROC","READ","REM","REPEAT","REPORT","RESTORE",\
          \"RETURN","RUN","STOP","COLOUR","TRACE","UNTIL","WIDTH","OSCLI",\
          \"","CIRCLE","ELLIPSE","FILL","MOUSE","ORIGIN","QUIT","RECTANGLE",\
          \"SWAP","SYS","TINT","WAIT","INSTALL","","PRIVATE","BY","EXIT"
        ENDIF
        LOCAL flag%, O$, i%, ch%
        IF ASC S$ = LENS$ AND ASC RIGHT$(S$,1) = 13 THEN
          ch% = ASC MID$(S$, 2)+256*ASC MID$(S$, 3)
          IF ch% O$=STR$ch%+" "
          S$ = MID$(S$, 4, LENS$-4)
          IF S$="" THEN =LEFT$(O$, LENO$-1)
        ENDIF
        FOR i%=1 TO LENS$
          IF ASC MID$(S$,i%)=34 flag%EOR=1
          IF flag% AND 1 THEN
            O$ += MID$(S$, i%, 1)
          ELSE
            ch% = ASCMID$(S$, i%)
            CASE TRUE OF
              WHEN ch% >= 17 AND ch% < 128:O$ += CHR$ch%
              WHEN ch% = &8D
                ch% = ASCMID$(S$,i%+1)
                lo% = ((ch% << 2) AND &C0) EOR ASC MID$(S$, i%+2)
                hi% = ((ch% << 4) AND &C0) EOR ASC MID$(S$, i%+3)
                O$+ = STR$(lo% + 256*hi%)
                i% += 3
              OTHERWISE O$ += Keywd$(ch% EOR 128)
            ENDCASE
          ENDIF
        NEXT i%
        =O$

Using FNdetokenise to list the error line

FNdetokenise is useful to list the source code of a line when one occurs.
Using FNdetokeniser to write a Quine

Using FNdetokenise it is possible to write a Quine, that is, a program which when run outputs its own listing:

        P% = PAGE
        WHILE (!P% AND &FFFFFF) <> &FFFF00
          line$ = ""
          FOR i% = P% TO P%+?P%-1
            line$ += CHR$?i%
          NEXT i%
          PRINT FNdetokenise(line$)
          P% += ?P%
        ENDWHILE
        END
