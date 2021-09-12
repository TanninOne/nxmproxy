# NXMProxy
Handles nexusmods.com download links and forwards them to a mod manager based on which game the download is for.

# Commandline

| Command | Arguments | Description | Example |
| --- | --- | --- | --- |
| help | - | Prints the command line commands | nxmproxy help |
| install | - | Sets up nxmproxy (this instance) in the system to handle nxm:// links | nxmproxy install |
| test | - | Check if nxmproxy (any instance!) is the system-wide handler for nxm:// links. This prints to stdout but also sets an exit code for use in tools and scripts | nxmproxy test |
| register | &lt;manager&gt; &lt;command&gt; | Configures a mod manager so it can be assigned to handle games | nxmproxy register vortex c:\program files\black tree gaming ltd\vortex\vortex.exe |
| deregister | &lt;manager&gt; | Drops a mod manager so it will no longer handle games | nxmproxy deregister NMM |
| pipe | &lt;manager&gt; &lt;pipe&gt; | Assigns a pipe name to the (already registered) manager. If set, when a download is started, nxmproxy will try to connect to that pipe, send the url (nothing else) and close the pipe afterwards. If that fails, the command from the register call is used, working on the assumption the manager is not running yet. As such you get no indication if you mistyped the pipe name!  | nxmproxy pipe vortex vortex_download |
| assign | &lt;manager&gt; &lt;game&gt; | Assigns a game to be handled by a manager. &lt;manager&gt; has to be the exact same id as used in the "register" call. &lt;game&gt; has to be the exact domain name as used in nexusmods.com/... links. &lt;game&gt; can also be _ (underscore) to make this manager the fallback handler for all games that have no manager assigned | nxmproxy assign vortex _ |
| url | &lt;nxm url&gt; | Starts a download | nxmproxy url nxm://somegame/mods/123/files/456 |

# Comparison to the ModOrganizer NXMHandler

NXMProxy is mostly a rewrite of NXMHandler with the goal to make it better to redistribute and integrate into different managers:

* NXMHandler has its own UI to configure it, which is convenient if used alone but doesn't integrate well with other managers. NXMProxy is configured either via
command line or directly through a configuration file.
* NXMHandler is built upon Qt and therefore requires >20MB of dlls if distributed separate or with a non-Qt application. NXMProxy has no dependencies.
* NXMHandler uses ini files for configuration, which is not a standardized file format. NXMProxy uses toml which is very very similar to ini but is an actual standard.

## Pipe

Usually forwarding a download to the mod manager involves opening the exe with the download link as a command line parameter. The Manager is then responsible to figure out if
another instance is already running and if so, use some form of IPC to send the url to the primary instance, then close.
Especially with mod managers requiring some form of VM (be it Java, Javascript, .Net, python) starting up the application causes a bit of delay.
NXMProxy can instead connect to a named pipe and send the url directly to the application listening on it to improve the response time.
