spec:
	busctl --xml-interface introspect  org.freedesktop.NetworkManager  /org/freedesktop/NetworkManager/Settings

code-gen:
	zbus-xmlgen system org.freedesktop.NetworkManager /org/freedesktop/NetworkManager/Settings
