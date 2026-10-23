# Learning Model: Evidence and Limits

## What the app currently measures

Daily records say only that a skill was checked in on a date. They do not record practice duration, task difficulty, feedback, task performance, or delayed retention. The current C/V accounts and category-specific level curves are therefore **game progression heuristics**, not validated estimates of ability. Their constants (`c`, `f`, `lambda`, `k`, `x0`) were not fitted to Soloup user data or to a study that measured the same tasks and outcomes. A check-in must not be interpreted as proof that proficiency increased by a known amount.

The app can validly report observable quantities: distinct practice days, current and longest streaks, recent practice frequency, and time since last practice. These describe practice behavior; they do not measure learning or transfer.

## What research supports

- Learning-curve shape is task- and measurement-dependent. Viering and Loog's review reports several curve families and concludes that no universal model is identified. Heathcote, Brown, and Mewhort found exponential fits beat power functions in their unaveraged response-time practice data; that is evidence for those tasks and outcomes, not a universal skill formula.
- Distributed practice can improve delayed retention, but the useful spacing depends on the material, schedule, and retention interval. A binary daily check-in does not reveal whether a session involved retrieval, feedback, or productive practice.
- Deliberate practice is associated with performance, but its explanatory contribution varies by domain. Macnamara, Hambrick, and Oswald's meta-analysis reported materially different estimates across games, music, sport, education, and professions. It does not support one category-wide coefficient for all skills.

## Model policy

1. Treat current levels as a **legacy, unvalidated activity-derived score** until a measured alternative is available; do not label it a scientific ability estimate.
2. Base achievements on directly observable events and explain their thresholds. Do not make goals such as 10,000 check-in days or tie real-world accomplishments to an unvalidated level threshold.
3. A future ability curve needs repeated, skill-specific performance observations using a stable measure, task difficulty, and assessment conditions. Fit candidate models to individual time series, compare fit and uncertainty, and show the measure and sample count. If there is insufficient data, show the observed measurements without extrapolating.
4. Keep daily check-ins lightweight. Performance assessments should be optional and separate; absence of assessments must never block recording practice.

## References

- Viering, T. & Loog, M. (2021). [The Shape of Learning Curves: a Review](https://arxiv.org/abs/2103.10948).
- Heathcote, A., Brown, S. & Mewhort, D. J. K. (2000). [The power law repealed: The case for an exponential law of practice](https://doi.org/10.3758/BF03212979). *Psychonomic Bulletin & Review*, 7, 185–207.
- Cepeda, N. J. et al. (2006). [Distributed practice in verbal recall tasks: A review and quantitative synthesis](https://doi.org/10.1037/0033-2909.132.3.354). *Psychological Bulletin*, 132(3), 354–380.
- Macnamara, B. N., Hambrick, D. Z. & Oswald, F. L. (2014). [Deliberate Practice and Performance in Music, Games, Sports, Education, and Professions: A Meta-Analysis](https://doi.org/10.1177/0956797614535810). *Psychological Science*, 25(8), 1608–1618.
