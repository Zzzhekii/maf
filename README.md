# Minimal Archive Format
Minimal Archive Format (MAF) is a lightweight and simple archive format.

This repository provides a reference MAF-compatible archiver written in Rust. 

# MAF's extension convention
The standard extension of MAF archives is '.maf'.
Compressed MAF archives should have a following extension: '.maf.<compression_algorithm>', where <compression_algorithm> is the compression algorithm (or it's usual abreviation) used to compress the archive. Example: '.maf.gz' could be a MAF archive compressed with the GZIP algorithm.

# Format Specification
MAF files always consist of an archive header, entry data list (EDL), optional MIME string, entry mapping list (EML), entry path list (EPL), and the  one right after the other (in that exact order).
### ALL NUMBERS USED IN THE FOLLOWING TABLES ARE LITTLE ENDIAN
### ALL TABLES COULD BE REPRESENTED AS A C STRUCT
### NO STRINGS ARE NULL-TERMINATED UNLESS THAT IS STATED SPECIFICALLY
## Archive header
Archive header starts at the very beginning of the MAF file.
Position and size in the following table are listed in bytes:
| Position | Size | Description                                                                                                     |
|----------|------|-----------------------------------------------------------------------------------------------------------------|
| 1 - 9    | 9    | Magic value (must always be 0xBADA556D6166676578 in hexadecimal (could be interpreted as 'ºÚUmafgex' in ASCII)) |
| 10       | 1    | MAF version number (this specification requires it to be 0)                                                     |
| 11 - 12  | 2    | NOT YET IMPLEMETED. MUST BE 0! MIME string length in bytes (optional, could be 0), maximum length is 65535      |
| 13 - 16  | 4    | Amount of entries                                                                                               |
| 17 - 24  | 8    | Size of the EPL in bytes                                                                                        |
| 25 - 40  | 16   | Size of the EDL in bytes                                                                                        |
| 41 - 64  | 24   | Reserved (must be 0)                                                                                            |

## Entry data list (EDL)
EDL consists of entry data (raw bytes) right after another.
Entry data is the contents of a file by it's matching entry path (in it's entry mapping).
Entry data can be any sequence of bytes

## Mime string (optional)
### CURRENTLY NOT STANDARDIZED - MUST NOT BE USED

## Entry mapping list (EML)
EML constists of entry mappings placed each right after another.
Entry mapping structure (position and size in the following table are listed in bytes):
| Position | Size | Description                                                                             |
|----------|------|-----------------------------------------------------------------------------------------|
| 1 - 8    | 8    | Beginning of the entry path offset in the EPL in bytes (first path's offset is 0)       |
| 9 - 24   | 16   | Beginning of the entry data offset in the EDL in bytes (first entry data's offset is 0) |
| 23 - 32  | 8    | Reserved (must be 0)                                                                    |

To figure out the length of an entry path (data), apart from the last one, MAF-compatible archiver needs to parse the following entry path (data).

## Entry path list (EPL)
EPL constists of paths encoded in ASCII written right after another.
A path is made up of multiple names (directories and file names) that are allowed to include alphanumeric characters (A-Z, a-z, and 0-9), spaces, blanks, mathematical symbols (+ - = | ~ ( ) < > { } \), punctuation marks (? , . ! ; : ' " / [ ]), and the following special characters: &, %, $, #, @, ^, *, and _.
### Note that the slash '/' character is treated as a path separater. It can be escaped by putting double slash ('//') instead.
Each path **must** have a matching entry mapping in the EML.
