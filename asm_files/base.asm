base    START   0

        BASE    2000
        LDB     #2000
        LDA     X

        NOBASE
        LDA     Y

        ORG     2800
X       WORD    42
Y       WORD    13