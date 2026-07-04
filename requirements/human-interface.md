# Human Interface

An interface allowing humans to interact with most critical functionality of Archiplan in the most convinient for a human format.

## Functionality

- View requirements as graph
- View detailed stress sessions as tables (stressor, attractor, breaking?, derived requirements' slugs)
- View architecture model as a versioned graph

## Interactive

User should see the progress of agent's work: new stress sessions, stressors, how spec evolves. All at high level, no need to see the details such as which methods agent called. 

## Hide technical terms from user-facing surfaces

Translate terminology such as NKP, Epistemic/Epistatic, corridors, attractors, etc into something normal developer understands.

## Human guidence

Для e2e решения сейчас не хватает сопровождения. Возник баг или ты видишь визуальный баг, вообще не понятно как его описать системе. Хочется сказать ему, что в такой-то ноде, рекваерменте проблема. 

Думаю. Скорее всего надо добавить пункт в реализацию, чтобы рантайм логи/ верстка HTML содержала id нод и рекваерментов
[2:17 PM]А то вот у меня вылез баг, а я понятия не имею как ему адекватно нт объяснить в чем проблема, кроме спеки. Он начинает не то фиксить часто