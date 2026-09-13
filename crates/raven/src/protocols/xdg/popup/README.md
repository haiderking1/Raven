# Panel popup grabs

A visible panel may open a grabbed menu using its most recent delivered pointer
press or release serial. The accepted grab consumes that authorization. Hovering
over a panel does not authorize a grab, and an exclusive keyboard layer takes
precedence. Keeping the release serial supports tray menus created after a D-Bus
round trip.

Panels with keyboard interactivity set to none receive pointer-only popup grabs.
Keyboard focus stays on the application, so keyboard input also stays there while
the menu is open. Giving these GTK 3 panels keyboard focus causes tray menu
failures and focus-dependent flicker. Keyboard-interactive panels and ordinary
window menus retain keyboard grabs.

The existing popup lifecycle releases only matching grabs when the popup closes
or its root becomes hidden. Rendering and buffer synchronization are unchanged.
