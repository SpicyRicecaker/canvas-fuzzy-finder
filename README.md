Have you ever thought about how much frustration you could save if you could just grep all module items in all your canvas courses for the specific item you need, instead of having to navigate 5+ times from your homepage to the module page for your course in question?

Introducing Canvas Fuzzy Finder (CFF). 

## To Use

Simply 
1. Clone this repository to `~/git`
2. Create a `.env` file inside `~/git/canvas-fuzzy-finder` that includes `TOKEN (which you can generate by going to settings > new access token on canvas), CANVAS_API_URL (e.g. canvas.youruni.edu/com), COURSE_IDS (comma + space separated integers you can find in the url of your browser), COURSE_NAMES (comma + space separated, corresponding to course ids)`
> I admit this step is convoluted but will improve with future releases
3. Run `cargo build --release` and save the resulting executable in `target/release` to start menu on windows or a launcher in macos!

## Supported OS
Windows, MacOS