# Minimal Archive Format
Minimal Archive Format (MAF) is a lightweight and simple archive format.
# Architechture
MAF files always consist of an archive header, optional MIME string, entry table, entry metadata, and the entries themselves (in that order).
An entry is made up of and entry header and the payload.
## Archive header
Archive header starts at the very beginning of the MAF file.
Position and size in the following table are listed in bytes:
| Position | Size | Description                                                                                                     |
|----------|------|-----------------------------------------------------------------------------------------------------------------|
| 1 - 9    | 9    | Magic value (must always be 0xBADA556D6166676578 in hexadecimal (could be interpreted as 'ºÚUmafgex' in ASCII)) |
| 10       | 1    | MAF version number (this specification requires it to be 0)                                                     |
| 11 - 12  | 2    | MIME string length in bytes (optional, could be 0)                                                              |
| 13 - 16  | 4    | Amount of entries                                                                                               |
