"""The party chat's messages, in the order they post and the frame each posts at (the design's schedule, as
prototypes/partychat.py makes it): data/messages.txt from data/credits_chat.txt.

    python3 make_data.py

The credits and the hype lines interleaved as partychat.py's messages(): a hype line first, then one after every
two credits, the last hype line closing the chat; a ^ line a reply, posted straight after the line above it. The
emoji are left out (just the scrolling text), and a message with nothing else goes too. They post
evenly from the layer's frame 0 until CLOSE (the stadium's frame 501, where its giant screen shows the CARDIAC): the
k-th at the first frame f with int(f / every) >= k, every = CLOSE / their count, as partychat.py's
int(k / every) + 1 shown.
"""
import math
import os
import re

here = os.path.dirname(os.path.abspath(__file__))
CLOSE = 457
# an emoji: a pictograph (and its variation selector), or a flag (two regional indicators): partychat.py's
EMOJI = re.compile("[\U0001F1E6-\U0001F1FF]{2}|[←-⯿☀-➿\U0001F000-\U0001FAFF]️?")


def messages():
    credits, hype, section = [], [], None
    for l in open(os.path.join(here, "credits_chat.txt"), encoding="utf-8"):
        l = l.strip()
        if l.startswith("# Hype"):
            section = hype
        if not l or l.startswith("#"):
            continue
        reply = l.startswith("^")
        name, text = (s.strip() for s in l.lstrip("?^ ").split("|", 1))
        text = " ".join(EMOJI.sub(" ", text).split())
        into = section if section is not None else credits
        if not text:
            continue
        if reply and into:
            into[-1].append((name, text))
        else:
            into.append([(name, text)])
    last = hype.pop()
    out = hype.pop(0)
    for i, c in enumerate(credits):
        out += c
        if i % 2 == 1 and hype:
            out += hype.pop(0)
    return out + [m for g in hype for m in g] + last


def main():
    ms = messages()
    every = CLOSE / len(ms)
    with open(os.path.join(here, "messages.txt"), "w", encoding="utf-8") as f:
        f.write("# the party chat's messages: the frame each posts at | handle | text (make_data.py)\n")
        for k, (name, text) in enumerate(ms):
            at = math.ceil(k * every - 1e-9)
            assert int(at / every) >= k and (at == 0 or int((at - 1) / every) < k)
            f.write(f"{at}|{name}|{text}\n")
    print(len(ms), "messages, one every %.1f frames" % every)


if __name__ == "__main__":
    main()
