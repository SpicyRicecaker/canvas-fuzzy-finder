ave you ever thought about how much frustration you could save if you could just grep all module items in all your canvas courses for the specific item you need, instead of having to navigate 5+ times from your homepage to the module page for your course in question?

Introducing Canvas Fuzzy Finder (CFF). 

# To Use

Simply 
1. Clone this repository to `~/git`
```shell
mkdir ~/git
cd ~/git
git clone https://github.com/SpicyRicecaker/canvas-fuzzy-finder
```
2. Create a `.env` file inside `~/git/canvas-fuzzy-finder` that includes `TOKEN (which you can generate by going to settings > new access token on canvas), CANVAS_API_URL (e.g. canvas.youruni.edu/com), COURSE_IDS (comma + space separated integers you can find in the url of your browser), COURSE_NAMES (comma + space separated, corresponding to course ids)`
```txt
echo "TOKEN=1234~exampleexampleexampleexampleexampleexampleexampleexampleexamplee
CANVAS_API_URL=https://canvas.someuniversity.edu
COURSE_IDS="1234567, 1234567, 1234567, 1234567, 1234567"
COURSE_NAMES="CS 101, CS 101, CS 101, CS 101, CS 101"" > .env
```
> I admit this step is convoluted but will improve with future releases
3. Run `cargo build --release` and save the resulting executable in `target/release` to start menu on windows or a launcher in macos!
```shell
cargo build --release
# the executable now in ./target/release/canvas-fuzzy-finder
```

# OS Dependencies
We currently support 1) Windows and 2) MacOS, and each platform has its own dependencies

## Windows
`winget` is a prerequisite as the general package manager. Install via [Microsoft Store](https://apps.microsoft.com/store/detail/app-installer/9NBLGGH4NNS1?hl=en-us&gl=us&rtc=1) if you don't have it already.
`git` is a prerequisite to clone the directory.
Powershell Core (`pwsh`) is a dependency for UTF-8 output to a file.
`fzf` is a dependency for interactive fuzzy search. 

```shell
winget install Git.Git Microsoft.Powershell junegunn.fzf 
```

## MacOS
`brew` is a prerequisite as the general package manager for macos. Install it at https://brew.sh/ if you don't have it already.
`fzf` is a dependency for interactive fuzzy search. 
`kitty` is a dependency for a fast terminal.

```shell
brew install fzf 
brew install kitty
```