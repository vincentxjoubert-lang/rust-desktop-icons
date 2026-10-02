## Nouveautés de la v0.7.13

### Fiabilité de vos fichiers
- **Désinstallation sûre** : tout le contenu des fences est remis sur le bureau, même si le fichier de configuration est abîmé. Le dossier de l'application n'est supprimé que lorsqu'il est vide.
- **Déplacements sûrs** : un déplacement interrompu (fichier ouvert dans un autre programme) ne peut plus faire perdre une partie d'un dossier.
- Déplacer un dossier dans lui-même (par exemple dans un portail qui pointe vers un de ses sous-dossiers) est refusé, au lieu de remplir le disque.

### Corrections
- Renommer un fichier en changeant seulement les majuscules (`photo` → `Photo`) fonctionne.
- Un message s'affiche si le dossier d'un onglet ne peut pas être créé ou si les réglages ne peuvent pas être enregistrés.
- Le journal d'erreurs ne grossit plus sans limite (archivé au-delà de 1 Mo).

### Performances
- **Moins de RAM** : environ 2× moins de mémoire au repos. L'image d'animation n'est gardée que lorsqu'une fence peut s'animer.
- **Rendu plus rapide** : environ 35 % plus rapide au survol des icônes, qui sont dessinées directement depuis la liste d'icônes de Windows.

### Sécurité
- La mise à jour automatique télécharge l'installeur sous un nom unique et le supprime après installation.
