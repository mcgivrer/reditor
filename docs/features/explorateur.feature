# language: fr
Fonctionnalité: Rafraîchissement de l'explorateur
  En tant qu'utilisateur de reditor
  Je veux que l'explorateur reflète les fichiers ajoutés, modifiés,
  renommés ou supprimés
  Afin de toujours voir l'état réel du système de fichiers

  Scénario: Un fichier ajouté sur le disque apparaît après rafraîchissement
    Étant donné un dossier a été ouvert explicitement
    Quand j'ajoute le fichier "nouveau.txt" directement sur le disque
    Et je rafraîchis l'explorateur
    Alors l'explorateur contient l'entrée "nouveau.txt"

  Scénario: Un fichier supprimé sur le disque disparaît après rafraîchissement
    Étant donné un fichier "temporaire.txt" contenant "temp"
    Et un dossier a été ouvert explicitement
    Quand je supprime le fichier "temporaire.txt" directement sur le disque
    Et je rafraîchis l'explorateur
    Alors l'explorateur ne contient pas l'entrée "temporaire.txt"

  Scénario: Un fichier renommé sur le disque est reflété après rafraîchissement
    Étant donné un fichier "ancien_nom.txt" contenant "contenu"
    Et un dossier a été ouvert explicitement
    Quand je renomme le fichier "ancien_nom.txt" en "nouveau_nom.txt" directement sur le disque
    Et je rafraîchis l'explorateur
    Alors l'explorateur ne contient pas l'entrée "ancien_nom.txt"
    Et l'explorateur contient l'entrée "nouveau_nom.txt"

  Scénario: Enregistrer sous rafraîchit automatiquement l'explorateur
    Étant donné un dossier a été ouvert explicitement
    Et un nouvel onglet vide
    Quand je tape "contenu"
    Et j'appuie sur "F10"
    Et j'appuie sur "Bas"
    Et j'appuie sur "Bas"
    Et j'appuie sur "Bas"
    Et j'appuie sur "Bas"
    Et j'appuie sur "Entrée"
    Et je saisis le nom de fichier "cree.txt" dans le dialogue
    Et je confirme le dialogue de fichier
    Alors l'explorateur contient l'entrée "cree.txt"

  Scénario: Reprendre le focus sur l'explorateur le rafraîchit
    Étant donné un dossier a été ouvert explicitement
    Quand j'ajoute le fichier "externe.txt" directement sur le disque
    Et j'appuie sur "F2"
    Alors l'explorateur contient l'entrée "externe.txt"
