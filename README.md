# Mindmaze
This is a little game we've made for Ludum Dare 45.

![](https://static.jam.vg/raw/801/72/z/2988c.png)

# Description

The code is obviously a mess. Be warned.

I thought I'd use Specs for everything, but it turns out that just reading about it doesn't give you enough experience to implement anything in a very short amount of time. So, it's dead code now.

The collision system was a bit of pain. I thought it would be easier to implement it if I keep walls in a tight grid pattern. It wasn't the first thing I wrong about.

For lighting, I've wept out some sort of a ray tracer. And it actually worked with just a little bit of tinkering.

The first attempts at implementing cinematics weren't very successful. It was either loading for too long or lagging mercilessly. The team was actually telling me to scrap them altogether, but I wanted to try a few quick changes. Loading frames on a fly, specifically the one that was about to be rendered, fixed cinematics for us. But probably not for other people, because memory usage jumps to 4 GB during the final cinematic. Well, that's just not how cinematics should be implemented in the first place.

The game took 644th place. Not to bad for our first attempt.

# Links
Ludum Dare 45 entry: https://ldjam.com/events/ludum-dare/45/mindmaze  
itch.io page: https://sigod.itch.io/mindmaze

