// ============================================================
// picker/linux/wayland/gnome/extension/extension.js
// Runs inside GNOME Shell. Lets XPicker take a screenshot (no prompt,
// no camera sound) and see where the cursor is. Once XPicker is
// uninstalled, removes itself and XPicker's shortcut at the next login.
// ============================================================

import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Shell from 'gi://Shell';
import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';

const NAME = 'com.quintixlabs.xpicker.Shell';
const PATH = '/com/quintixlabs/xpicker/Shell';
const INTERFACE = `
<node>
  <interface name="com.quintixlabs.xpicker.Shell">
    <method name="Screenshot">
      <arg type="i" direction="in" name="x"/>
      <arg type="i" direction="in" name="y"/>
      <arg type="i" direction="in" name="width"/>
      <arg type="i" direction="in" name="height"/>
      <arg type="s" direction="in" name="file"/>
    </method>
    <method name="Cursor">
      <arg type="d" direction="out" name="x"/>
      <arg type="d" direction="out" name="y"/>
    </method>
  </interface>
</node>`;

/** Only XPicker itself may take screenshots: anything else would be a way
 *  around GNOME's own screenshot permission. */
function fromXPicker(sender) {
  try {
    const [pid] = Gio.DBus.session
      .call_sync('org.freedesktop.DBus', '/org/freedesktop/DBus', 'org.freedesktop.DBus', 'GetConnectionUnixProcessID', new GLib.Variant('(s)', [sender]), null, Gio.DBusCallFlags.NONE, 1000, null)
      .deepUnpack();
    const program = GLib.path_get_basename(GLib.file_read_link(`/proc/${pid}/exe`));
    return program.toLowerCase() === 'xpicker';
  } catch {
    return false;
  }
}

class Service {
  /** The area (in GNOME's own units) as a PNG at the screen's real resolution. */
  ScreenshotAsync([x, y, width, height, file], invocation) {
    if (!fromXPicker(invocation.get_sender())) {
      invocation.return_dbus_error('org.freedesktop.DBus.Error.AccessDenied', 'Only XPicker can take screenshots here.');
      return;
    }
    try {
      const stream = Gio.File.new_for_path(file).replace(null, false, Gio.FileCreateFlags.PRIVATE, null);
      new Shell.Screenshot().screenshot_area(x, y, width, height, stream, (shooter, result) => {
        try {
          shooter.screenshot_area_finish(result);
          stream.close(null);
          invocation.return_value(null);
        } catch (e) {
          invocation.return_dbus_error('org.freedesktop.DBus.Error.Failed', e.message);
        }
      });
    } catch (e) {
      invocation.return_dbus_error('org.freedesktop.DBus.Error.Failed', e.message);
    }
  }

  Cursor() {
    const [x, y] = global.get_pointer();
    return [x, y];
  }
}

// ------------------------------------------------------------
// After XPicker is uninstalled
// ------------------------------------------------------------

const SHORTCUTS = 'org.gnome.settings-daemon.plugins.media-keys';
const SHORTCUT_PATH = '/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/xpicker/';

/** Settings by name, or null if this system doesn't have them. */
function settings(schema, path) {
  if (!Gio.SettingsSchemaSource.get_default().lookup(schema, true)) return null;
  return path ? new Gio.Settings({ schema_id: schema, path }) : new Gio.Settings({ schema_id: schema });
}

/** XPicker saves where it is next to this file. True when that's gone. */
function xpickerGone(folder) {
  try {
    const [, bytes] = GLib.file_get_contents(GLib.build_filenamev([folder, 'xpicker-path']));
    const program = new TextDecoder().decode(bytes).trim();
    return program !== '' && !GLib.file_test(program, GLib.FileTest.EXISTS);
  } catch {
    return false; // not saved (yet): leave everything as it is
  }
}

/** Removes the shortcut, this extension from GNOME's list, and its files. */
function cleanUp(uuid, folder) {
  const keys = settings(SHORTCUTS);
  if (keys) {
    keys.set_strv('custom-keybindings', keys.get_strv('custom-keybindings').filter((p) => p !== SHORTCUT_PATH));
  }
  const entry = settings(`${SHORTCUTS}.custom-keybinding`, SHORTCUT_PATH);
  for (const key of ['name', 'command', 'binding']) entry?.reset(key);

  const shell = settings('org.gnome.shell');
  if (shell) shell.set_strv('enabled-extensions', shell.get_strv('enabled-extensions').filter((id) => id !== uuid));

  for (const name of ['metadata.json', 'extension.js', 'xpicker-path']) {
    try {
      Gio.File.new_for_path(GLib.build_filenamev([folder, name])).delete(null);
    } catch {}
  }
  try {
    Gio.File.new_for_path(folder).delete(null);
  } catch {}
}

export default class XPickerExtension extends Extension {
  enable() {
    if (xpickerGone(this.path)) {
      cleanUp(this.uuid, this.path);
      return;
    }
    this._service = Gio.DBusExportedObject.wrapJSObject(INTERFACE, new Service());
    this._service.export(Gio.DBus.session, PATH);
    this._name = Gio.bus_own_name_on_connection(Gio.DBus.session, NAME, Gio.BusNameOwnerFlags.NONE, null, null);
  }

  disable() {
    if (!this._service) return;
    Gio.bus_unown_name(this._name);
    this._service.unexport();
    this._service = null;
  }
}
