# taskmaster

## Description
Job controller in rust, inspired by supervisor.

## Usage
### Configuration file example
- name : str 
 command : str
 numprocs : i32
 autostart : bool
 autorestart : true | false | unexpected

### Getting started

start server : 

cargo run --bin server -- CONFIG_FILE

start client

cargo run --bin client 

### client commands

Start / Stop / Restart

Status

Reload

## Roadmap
a debattre : 
utilisation de nix pour la gestion des signaux ?

** SERVER ** :
- [] fix : restart proc (update method)
- [] le programme doit etre redemarre: toujours/jamais/seulement sur unexpected code return (a voir comment c'est gere avec autorestart)
- [] code retour attendu du processus
- temps pedant lequel le programme doit reste en vie pour etre considere succesfully started
- combien de tentative de restart avant d'etre aborded.
- avec quel signal on doit stop le programme 
- combien de temps attendre avant de considerer un programme successfully stopped (bon code de retour)
- stdout/stderr du process
- variable d'environnement to set avant l'execution du process
- set le repertoire courant avant l'execution d'un programme
- umask to set avant l'execution du programme


_______________________________________________________________________

** CLIENT ** :
[] - help command 
[] - FIX : command with space return an error
[] - dispatch displayed outputs / command
    [] - Reload : 
        [] - display : Really reload the remote taskmaster server process y/N ?
        [] - get y /n : n -> \n\r  | y -> Restarted taskmaster server (ne verifie pas la reponse)
