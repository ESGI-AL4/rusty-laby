# rusty-laby

### Enoncé
https://github.com/haveneer-training/sauve_qui_peut

### Lancer le serveur
```
./server run
```
### Lancer le serveur en mode debug
```
./server --debug run
```
### Lancer le serveur en mode debug avec 1 seul jour par équipe
```
.\server.exe --debug run --team-size 1
```

### Decoder un radarView
```
cargo run --bin radar_view decode ieysGjGO8papd/a
```

### Lancer une partie
```
cargo run --bin rusty-laby -- params?

params:
--team-size n : The team size (default to 3)
--ui : enables the ui (only for team-size 1)
```