## in-memory addresses guesses

#### 1.66A
- 0081d960 runs the hkWorldCInfo initializer
- 004836d0 is the hkWorldCInfo initializer
- 00f41cc4 is a float32 value of "-9.81", likely used through out the game including as a gravity value

- 009e2721 loads normal mode gravity modifier

- based on the findings of https://github.com/MeFisto94
	- 008345e0 seems to be executed on vehicle spawn, vehicle physics is applied during that
	- 008348a1 is around where hc physics can be forced on/off
