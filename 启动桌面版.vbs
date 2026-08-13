Option Explicit

Dim shell, fso, root, scriptPath, file, cmd
Set shell = CreateObject("WScript.Shell")
Set fso = CreateObject("Scripting.FileSystemObject")
root = fso.GetParentFolderName(WScript.ScriptFullName)

' Find the sibling launcher without embedding a non-ASCII filename. This
' keeps the hidden VBS launcher working even when its encoding is changed.
scriptPath = ""
For Each file In fso.GetFolder(root).Files
  If LCase(fso.GetExtensionName(file.Name)) = "cmd" Then
    scriptPath = file.Path
    Exit For
  End If
Next

If scriptPath = "" Then
  MsgBox "Launcher CMD file was not found beside this VBS file.", vbCritical, "My Codex"
  WScript.Quit 1
End If

cmd = "cmd.exe /d /s /c call " & Chr(34) & scriptPath & Chr(34)
shell.CurrentDirectory = root
shell.Run cmd, 0, False
