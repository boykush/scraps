#[[Emit/Static Site]]

Every scrap page that has at least one connection draws its direct
neighbours as an inline SVG, above the backlinks and links lists. The
drawing is generated at build time — no JavaScript is fetched or run for
it, and no extra file is emitted.

Direction is carried by position rather than colour: backlinks sit on the
left arc, links on the right, and scraps linked both ways sit on the
vertical. Arrowheads always point at the scrap being referenced. Colours
come from the same tokens as the rest of the page, so the graph follows
[[Reference/Static Site/Color Scheme]] without a redraw.

A hub scrap can carry hundreds of connections, past which the labels stop
being readable. At most 18 neighbours are drawn, chosen round-robin across
the three directions so a scrap with many backlinks and one outgoing link
still shows that link; the remainder is reported as a `+N more` count.
