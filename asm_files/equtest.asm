equtst      START   0
            LDA     #HALF
            LDA     #BEF


HALF        EQU     LEN/2
LEN         EQU     LAST-BUF
BEF         EQU     BUF-1

BUF         RESB    1000
LAST        EQU     *