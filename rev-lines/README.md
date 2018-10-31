# rev-lines

Lazily reads lines of a given UTF-8 text file backwards, just like `tail -r`.

## Usage

```bash
# reads 10 lines backwords
rev-lines $FILE | head -n 10
```

**Note**: If the file ends with a newline, the first line printed will be empty.
