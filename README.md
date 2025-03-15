# Fzz a fuzzy finder for the terminal

Fzz is a fuzzy finder match like fzf, but blazingly fast.


## Usage
Using fzz is straight forward

```bash
# This will split the string by whitespace.
# this is nice if you want to create a simple select menu
echo "a b c" | fzz -d ' '

# This simply splits by comma,
echo "a,b,c" | fzz -d ','

# This works the same as fzf, 
# pipes ls in to fzz and allows for fuzzy search
ls | fzz

# by default fzz uses case-insensative search
# if you want to make it a case sens search use 
ls | fzz -c
```



