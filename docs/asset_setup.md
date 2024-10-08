
# Asset Manager System setup 

## Asset Manager domain
### This will cover what files the asset manager will manipulate and how it will organize them

```Definition 
Asset : a file, binary or ascii, that is used to hold **game data** information that does not manipulate the state of the game.
```

Assets must always be stored within an asset folder of any game dll that is utilizing it. For a game to gain access to an asset, the asset folder in which the asset is held in must be made visible to the Asset Manager system. This allows for multiple asset folders that will be stored as separate asset bundles that the asset manager will load when loading the game.
```
game_dir
|
|__ asset_folder_1
|
|__ asset_folder_2
```

Any files that are loaded from the asset folders post registration will cause for unknown functionality during release usage, as the folder will no longer be available when 


