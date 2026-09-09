# language: fr
Fonctionnalité: Compilation d'un projet Java
  En tant qu'utilisateur de reditor éditant un projet Java
  Je veux compiler mon projet sans quitter l'éditeur
  Afin de vérifier rapidement que mon code compile

  Scénario: Le menu Compiler apparaît pour un projet contenant du Java
    Étant donné un fichier "Main.java" contenant "public class Main {}"
    Et un dossier a été ouvert explicitement
    Alors le menu "Compiler" est présent

  Scénario: Le menu Compiler est absent pour un projet sans fichier Java
    Étant donné un dossier a été ouvert explicitement
    Alors le menu "Compiler" est absent

  Scénario: Configurer la compilation affiche les JDK détectés
    Étant donné un fichier "Main.java" contenant "public class Main {}"
    Et un dossier a été ouvert explicitement
    Quand j'appuie sur "F6"
    Alors une fenêtre de configuration de compilation est affichée

  Scénario: Sélectionner un JDK dans le dialogue met à jour le message de statut
    Étant donné un fichier "Main.java" contenant "public class Main {}"
    Et un dossier a été ouvert explicitement
    Quand j'appuie sur "F6"
    Et je sélectionne le premier JDK détecté dans le dialogue de compilation
    Alors aucune fenêtre de configuration de compilation n'est affichée
    Et le message de statut contient "JDK sélectionné"

  Scénario: Compiler un projet Java valide réussit
    Étant donné un fichier "Main.java" contenant "public class Main { public static void main(String[] args) { int x = 1 + 1; } }"
    Et un dossier a été ouvert explicitement
    Quand j'appuie sur "F6"
    Et je sélectionne le premier JDK détecté dans le dialogue de compilation
    Et j'appuie sur "F5"
    Alors le message de statut contient "Compilation réussie"
    Et une fenêtre de résultat de compilation "réussie" est affichée

  Scénario: Compiler un projet Java invalide signale l'échec
    Étant donné un fichier "Main.java" contenant "ceci n'est pas du java valide"
    Et un dossier a été ouvert explicitement
    Quand j'appuie sur "F6"
    Et je sélectionne le premier JDK détecté dans le dialogue de compilation
    Et j'appuie sur "F5"
    Alors le message de statut contient "Échec de la compilation"
    Et une fenêtre de résultat de compilation "échouée" est affichée
