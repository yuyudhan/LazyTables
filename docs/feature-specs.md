# Tech stack.
- Terminal-based application
- Built with Go
- Fast, lightweight, and efficient
- We'll support PostgreSQL, MySQL, and SQLite for now, with plans to extend to other database systems in the future.

# Requirements:
We're building a Lazygit alternative for managing SQL tables that supports all major databases and can be easily extended. Inspired by TablePlus, our goal is to create a simple, elegant terminal-based tool that frees users from the clumsiness of PGadmin and phpMyAdmin, making SQL management more efficient for developers worldwide.

When the user opens LazyTables (using 'lazytables' command in installed app or 'air' command in dev mode) in their terminal they will see a left side, left side will have three boxes one over the other, first will be 'connections', second will be list of databases, third on the most bottom will be list of tables. Left side will be smaller in width like a sidebar.

On the main area, or the right pane, we'll have two things, one will be to edit and execute sql queries, on the bottom part of the right side, we'll have the results display, either table read output or query status.

There will be global keybindings to toggle the visibility and take focus.
Pressing small `c` will take the focus into the connections, pressing capital `C` will toggle the visibility of the connections box.
Similarly pressing `d` will take focus into databases list box, pressing capital `D` will toggle its visibility.
Similarly pressing small `t` will take focus into tables list box, pressing capital `T` will toggle it's visibility.
Pressing q will take focus into query box, pressing capital Q will toggle it's visibility.
Pressing o will take focus into output box, pressing capital O will toggle it's visibility.

:> [!NOTE]
> Centralize global keybindings for toggling view visibility and setting focus. Ensure consistent, non-overlapping keybindings across all views.

# Left sidebar
This will occupy 20% width of the screen, this need not be a separate view in itself. But more like a UI look and feel.
The left sidebar will consist of three boxes each occupying 30% height of the sidebar area. First box will be connections box, second box will list databases, third box will list the tables. Along with viewing and listing each of these boxes will also allow editing adding of respective objects.

Pressing vim key bindings j and k to move down and up in that list.
Pressing a when focussed into connections view, should open a dialog to add connection.
Pressing d when focussed into an item in connections list shall remove the connection after confirming y or n. This confirmation will be taken in the status bar like Tmux.

## Connections box.

## Databases box.
Just navigation for now.
Pressing 's' will select the database currently in focus on the list.

## Tables box
Just navigation for now.
Pressing 's' will select the table currently in focus on the list, the tables list shall be updated as per the database selected in the database box. If database is not selected display message in this box showing 'No database selected'

:> [!NOTE]
> The list of connections will be stored in the OS-designated application data directory.
> - **macOS:** Typically in `~/Library/Application Support/LazyTables`
> - **Linux:** Usually in `~/.config/LazyTables` (or an equivalent directory based on distribution guidelines)

# Main area (80% width of the screen visible on Right of the left sidebar)
Main area will occupy 80% of screen width.
It'll have two boxes, one top 20% shall be query box.
This query box will get focussed when q is pressed, its view will be toggled when pressing Q.

## Output box
Bottom 80% of the main area shall be the output box, it'll either present the output of the last query in tabular format or display an empty table if the query did not output a table.
When displaying tables, we have to basically allow vim like navigation using h,j,k,l to navigate through the table using cells.
We don't want to allow editing cells in the table for now, but that we'll build later on. Right now we'll focus only on the view part.

# Status bar
The status bar should follow the theme like crated by powerlevel10k theme.
This bar will have several features. At any moment in time these things shall be displayed on the status bar.

## Left side of bottom status bar.
- The left most area of the box will display which box is focussed on, example connections, databases, tables, query or output boxes.
- The second area of the box adjoining the first area will display. The connection name which is active, or say no connection active.
- The third element will display which database active, this will only be visible when a connection is active, if no db active it should say 'No DB active'
- Fourth element will display which table active, should be visible only when a database is selected. When no table is active or selected, then it should basically say 'No table active'.

## Right side of the bottom status bar.
Right most thing shall be the date, second right most shall be current time in seconds.

# Notifications view.

This will be on the top right, they will be displayed as transient notifications, they will be visible for 3 seconds before vanishing. If multiple modules send the notifications, they will be stacked and vanish when each of them finish their 3 second lifetime, this 3 second shall be from the config file.

The notifications shall be of different colors, as per the notification type, like error, information, the module should also expose.

# Other details

- Automatically adjust the layout when the terminal resizes.
- Maintain the 20%/80% split and reflow UI components gracefully.
- Follow a Powerlevel10k-inspired theme for the status bar.
- Plan for advanced features such as cell editing in the output view. Need not worry about this right now.
- Consider adding a detailed query log or debugging panel later. Need not worry about this right now.
- Provide clear focus indicators for active views.

