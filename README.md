# Termset

The idea behind termset, is to provide a barebones interface for interacting with the terminal. 
It is not in any way supposed to be a complete TUI library, but is intended to be used as an 
alternative backend to something like ncurses.

I have no intention of making this compatible with anything other than Linux, however, I believe
this should actually work just fine on Windows right now, though I do not care.

### Desired Functionality

- Easily clear the terminal, storing the previous contents
- Acquire information about the terminal reactively
    - terminal size
    - mouse cursor location (as terminal coordinates)
- Easily switch to a raw input mode, allowing the user to 
    - write a buffer to a specific location on the terminal
    - control the cursor (and keep track of its location)
- Easily restore the terminal to its previous state (`Termset::restore(self)`)

This is (pretty much) an exhaustive list. I intend to make a more complex TUI library on top of 
this, but I know a lot of people want to do that kind of thing on their own, so I'm hoping this
can act as an easy-to-learn backend for people who want to make their own TUI (for fun).