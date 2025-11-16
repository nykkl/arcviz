# Arcviz
Try its web deployment [here](https://nykkl.github.io/arcviz). (You'll probably need a chromium-based browser to open/save files.)

## Web app
This application can also be used as a web app.
```sh
nix build .#arcviz-web
```
This will produce a directory `./result/share` that you can serve to web clients.  

Unfortunately this will not work properly on firefox since they don't seem to think it necessary to provide some way of accessing the filesystem, hence you would not be able to save your work. (Same with Safari.)
So I recommend using chromium/chrome but other browsers may also work.  
The actual feature required is the `File System Access` api (distinct from `File System` api), but for current data on browser compatibility you may want to refer to [this](https://developer.mozilla.org/en-US/docs/Web/API/Window/showOpenFilePicker#browser_compatibility).  

## Building

### nix
```sh
nix build
```
The ouput will be linked to by `./result`.  
You may instead run `nix run` to run it directly or `nix install` to install it install it into you user profile.

### Building distributables (AppImage, ..)
```sh
nix develop -i
```
In `./electron-wrapper/dist/` you should now find artefacts (e.g. an AppImage) that you can distribute to other (e.g. non nix) systems.  
Make sure first that `./electron/app` does not exist yet and that you are in the projects root directory.

## Development

### Serving the web app
For development you might want to run the app in the browser.
The easiest way to do that is to just build with:
```sh
nix build .#arcviz-web
```
and then serve the `./result/share` directory.
To do this i use usually use `miniserve`.
But you have to be careful because the browser usually caches the data which would lead to the site not being updated to reflect you code changes after you run it the first time.

#### Option 1
I'm no expert at this but the most reliable way to avoid that seems to be to use the `Clear-Site-Data` header like this:
```sh
miniserve ./result/share --header 'Clear-Site-Data: "*"'
```
But this will make loading **VERY** slow.

#### Option 2
An alternative is to just not cache in the first place with `Cache-Control`:
```sh
miniserve ./result/share --header 'Cache-Control: no-cache, no-store'
```
But this only works if you do it from the start since it doesn't delete any existing cache, which is very annoying if you forget once.

#### Option 3
The third way would be to just disable caching in your browsers developer tools.
This probably works the best, but the downside here is that you have to do it manually everytime.

I would probably go with option 2 and if you mess it up you can run option 1 once to fix it and then continue with option 2.
You will still have to restart miniserve after rebuilding though.
