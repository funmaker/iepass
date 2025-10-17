# Notes

# Pico8
- decimal numeric literals are parsed into f64 and then truncates into Q16.16, wrapping on overflows
- hex/binary numeric literals parsed directly into Q16.16, truncated, wrapping on overflows
- tostr displays up to 4 decimal places, truncated
- arithmetic wraps on overflow
- modulo is euclidean (corresponds to \ operator)
- left shift is logical



| Id  | Character            | Escaped | Unicode | Name                           |
| --- | -------------------- | ------- | ------- | ------------------------------ |
| 0   | \<Null>              | \\0     | U+0000  | Terminate printing             |
| 1   | ¹                    | \\*     | U+00B9  | Repeat next character          |
| 2   | ²                    | \\#     | U+00B2  | Draw solid background          |
| 3   | ³                    | \\-     | U+00B3  | Move cursor horizontally       |
| 4   | ⁴                    | \\|     | U+2074  | Move cursor vertically         |
| 5   | ⁵                    | \\+     | U+2075  | Move cursor                    |
| 6   | ⁶                    | \\^     | U+2076  | Special command                |
| 7   | ⁷                    | \\a     | U+2077  | Audio command                  |
| 8   | ⁸                    | \\b     | U+2078  | Backspace                      |
| 9   | \<Tab>               | \\t     | U+0009  | Tab                            |
| 10  | \<End of Line>       | \\n     | U+000A  | Newline                        |
| 11  | ᵇ                    | \\v     | U+1D47  | Decorate previous character    |
| 12  | ᶜ                    | \\f     | U+1D9C  | Set foreground color           |
| 13  | \<Carriage Return>   | \\r     | U+000D  | Carriage return                |
| 14  | ᵉ                    | \\14    | U+1D49  | Switch font defined at 0x5600  |
| 15  | ᶠ                    | \\15    | U+1DA0  | Switch font to default         |
| 16  | ▮                    |         | U+25AE  | Vertical rectangle             |
| 17  | ■                    |         | U+25A0  | Filled square                  |
| 18  | □                    |         | U+25A1  | Hollow square                  |
| 19  | ⁙                    |         | U+2059  | Five dot                       |
| 20  | ⁘                    |         | U+2058  | Four dot                       |
| 21  | ‖                    |         | U+2016  | Pause                          |
| 22  | ◀                    |         | U+25C0  | Back                           |
| 23  | ▶                    |         | U+25B6  | Forward                        |
| 24  | 「                    |         | U+300C  | Japanese starting quote        |
| 25  | 」                    |         | U+300D  | Japanese ending quote          |
| 26  | ¥                    |         | U+00A5  | Yen sign                       |
| 27  | •                    |         | U+2022  | Interpunct                     |
| 28  | 、                    |         | U+3001  | Japanese comma                 |
| 29  | 。                    |         | U+3002  | Japanese full stop             |
| 30  | ゛                    |         | U+309B  | Japanese dakuten               |
| 31  | ゜                    |         | U+309C  | Japanese handakuten            |
| 32  | \<Space>             |         | U+0020  | Space                          |
| 33  | !                    |         | U+0021  | !                              |
| 34  | "                    | \\"     | U+0022  | Double quote                   |
| 35  | #                    |         | U+0023  | Number sign                    |
| 36  | $                    |         | U+0024  | Dollar sign                    |
| 37  | %                    |         | U+0025  | Percent sign                   |
| 38  | &                    |         | U+0026  | Ampersand                      |
| 39  | '                    | \\'     | U+0027  | Single quote                   |
| 40  | (                    |         | U+0028  | (                              |
| 41  | )                    |         | U+0029  | )                              |
| 42  | *                    |         | U+002A  | *                              |
| 43  | +                    |         | U+002B  | +                              |
| 44  | ,                    |         | U+002C  | ,                              |
| 45  | -                    |         | U+002D  | -                              |
| 46  | .                    |         | U+002E  | .                              |
| 47  | /                    |         | U+002F  | /                              |
| 48  | 0                    |         | U+0030  | 0                              |
| 49  | 1                    |         | U+0031  | 1                              |
| 50  | 2                    |         | U+0032  | 2                              |
| 51  | 3                    |         | U+0033  | 3                              |
| 52  | 4                    |         | U+0034  | 4                              |
| 53  | 5                    |         | U+0035  | 5                              |
| 54  | 6                    |         | U+0036  | 6                              |
| 55  | 7                    |         | U+0037  | 7                              |
| 56  | 8                    |         | U+0038  | 8                              |
| 57  | 9                    |         | U+0039  | 9                              |
| 58  | :                    |         | U+003A  | :                              |
| 59  | ;                    |         | U+003B  | ;                              |
| 60  | <                    |         | U+003C  | <                              |
| 61  | =                    |         | U+003D  | =                              |
| 62  | >                    |         | U+003E  | >                              |
| 63  | ?                    |         | U+003F  | ?                              |
| 64  | @                    |         | U+0040  | @                              |
| 65  | A                    |         | U+0041  | A                              |
| 66  | B                    |         | U+0042  | B                              |
| 67  | C                    |         | U+0043  | C                              |
| 68  | D                    |         | U+0044  | D                              |
| 69  | E                    |         | U+0045  | E                              |
| 70  | F                    |         | U+0046  | F                              |
| 71  | G                    |         | U+0047  | G                              |
| 72  | H                    |         | U+0048  | H                              |
| 73  | I                    |         | U+0049  | I                              |
| 74  | J                    |         | U+004A  | J                              |
| 75  | K                    |         | U+004B  | K                              |
| 76  | L                    |         | U+004C  | L                              |
| 77  | M                    |         | U+004D  | M                              |
| 78  | N                    |         | U+004E  | N                              |
| 79  | O                    |         | U+004F  | O                              |
| 80  | P                    |         | U+0050  | P                              |
| 81  | Q                    |         | U+0051  | Q                              |
| 82  | R                    |         | U+0052  | R                              |
| 83  | S                    |         | U+0053  | S                              |
| 84  | T                    |         | U+0054  | T                              |
| 85  | U                    |         | U+0055  | U                              |
| 86  | V                    |         | U+0056  | V                              |
| 87  | W                    |         | U+0057  | W                              |
| 88  | X                    |         | U+0058  | X                              |
| 89  | Y                    |         | U+0059  | Y                              |
| 90  | Z                    |         | U+005A  | Z                              |
| 91  | [                    |         | U+005B  | [                              |
| 92  | \                    | \\\     | U+005C  | \                              |
| 93  | ]                    |         | U+005D  | ]                              |
| 94  | ^                    |         | U+005E  | Caret                          |
| 95  | _                    |         | U+005F  | Underscore                     |
| 96  | `                    |         | U+0060  | Backtick                       |
| 97  | a                    |         | U+0061  | a                              |
| 98  | b                    |         | U+0062  | b                              |
| 99  | c                    |         | U+0063  | c                              |
| 100 | d                    |         | U+0064  | d                              |
| 101 | e                    |         | U+0065  | e                              |
| 102 | f                    |         | U+0066  | f                              |
| 103 | g                    |         | U+0067  | g                              |
| 104 | h                    |         | U+0068  | h                              |
| 105 | i                    |         | U+0069  | i                              |
| 106 | j                    |         | U+006A  | j                              |
| 107 | k                    |         | U+006B  | k                              |
| 108 | l                    |         | U+006C  | l                              |
| 109 | m                    |         | U+006D  | m                              |
| 110 | n                    |         | U+006E  | n                              |
| 111 | o                    |         | U+006F  | o                              |
| 112 | p                    |         | U+0070  | p                              |
| 113 | q                    |         | U+0071  | q                              |
| 114 | r                    |         | U+0072  | r                              |
| 115 | s                    |         | U+0073  | s                              |
| 116 | t                    |         | U+0074  | t                              |
| 117 | u                    |         | U+0075  | u                              |
| 118 | v                    |         | U+0076  | v                              |
| 119 | w                    |         | U+0077  | w                              |
| 120 | x                    |         | U+0078  | x                              |
| 121 | y                    |         | U+0079  | y                              |
| 122 | z                    |         | U+007A  | z                              |
| 123 | {                    |         | U+007B  | {                              |
| 124 | \|                   |         | U+007C  | Vertical bar                   |
| 125 | }                    |         | U+007D  | }                              |
| 126 | ~                    |         | U+007E  | Tilde                          |