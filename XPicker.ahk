#Requires AutoHotkey v2.0
#SingleInstance Force

;==============================================================
; XPicker — Quintix
;==============================================================
; Copyright (c) 2025 Quintix
; Licensed under the GNU General Public License v3.0
; License: https://github.com/QuintixLabs/XPicker/blob/master/LICENSE
;
; Issues:      https://github.com/QuintixLabs/XPicker/issues
; Contributions: https://github.com/QuintixLabs/XPicker/pulls
;
; You may use, modify, and redistribute under GPLv3.
; Do not sell or claim this work as your own.
;==============================================================


; Global variables :)
magnificationFactor := 9
magnifierSize := 150
colorBoxSize := 60
borderWidth := 0
textHeight := 25
colorFormatIndex := 1
lastFormatSwitchTime := 0

; Initialize
A_TrayMenu.Add("Exit", (*) => ExitApp())
OnExit(RestoreCursors)
SetSystemCursor("IDC_CROSS")

magnifierGui := Gui("-Caption +ToolWindow +AlwaysOnTop +E0x20")
magnifierGui.BackColor := "FFFFFF"
magnifierGui.SetFont("s10 bold", "Segoe UI")
magnifierGui.MarginX := 0
magnifierGui.MarginY := 0

colorText := magnifierGui.Add("Text", "x0 y" . (magnifierSize - textHeight) . " w" . magnifierSize . " h" . textHeight . " Center BackgroundWhite", "#000000")
colorText.SetFont("s10 bold")

magnifierHwnd := magnifierGui.Hwnd
magnifierDC := DllCall("GetDC", "Ptr", magnifierHwnd, "Ptr")
magnifierBitmap := DllCall("CreateCompatibleBitmap", "Ptr", magnifierDC, "Int", magnifierSize, "Int", magnifierSize, "Ptr")
memDC := DllCall("CreateCompatibleDC", "Ptr", magnifierDC, "Ptr")
DllCall("SelectObject", "Ptr", memDC, "Ptr", magnifierBitmap)
magnifierGui.Show("w" . magnifierSize . " h" . magnifierSize . " Hide")

; Hotkeys
RButton::CatchColor()
^RButton::CatchColor()
Esc::ExitApp()

Loop {
    CoordMode("Mouse", "Screen")
    CoordMode("Pixel", "Screen")
    MouseGetPos(&x, &y)
    
    ; get the exact pixel color under the cursor
    color := PixelGetColor(x, y, "RGB")
    color := "0x" . SubStr(color, 3)
    
    currentTime := A_TickCount
    if ((GetKeyState("LControl", "P") || GetKeyState("RControl", "P")) && (currentTime - lastFormatSwitchTime > 240)) {
        colorFormatIndex := Mod(colorFormatIndex, 5) + 1
        lastFormatSwitchTime := currentTime
    }
    
    colorMessage := FormatColor(color, colorFormatIndex)
    
    UpdateMagnifier(x, y, color, colorMessage)
    
    Sleep(10)
}

; Update the magnifier
UpdateMagnifier(x, y, currentColor, colorValue) {
    global magnifierSize, magnificationFactor, colorText, magnifierDC, memDC, magnifierHwnd, textHeight
    
    screenWidth := A_ScreenWidth
    screenHeight := A_ScreenHeight
    
    offsetX := magnifierSize // 2
    offsetY := magnifierSize // 2
    
    magnifierX := x - offsetX
    magnifierY := y - offsetY - 100
    
    if (magnifierX < 0)
        magnifierX := 0
    else if (magnifierX + magnifierSize > screenWidth)
        magnifierX := screenWidth - magnifierSize
        
    if (magnifierY < 0) {
        magnifierY := y + 50
        
        if (magnifierY + magnifierSize > screenHeight)
            magnifierY := screenHeight - magnifierSize
    }
    else if (magnifierY + magnifierSize > screenHeight)
        magnifierY := screenHeight - magnifierSize
    
    screenDC := DllCall("GetDC", "Ptr", 0, "Ptr")
    DllCall("gdi32\SetStretchBltMode", "Ptr", memDC, "Int", 3)  ; COLORONCOLOR (3) for better quality
    
    DllCall("gdi32\BitBlt"
        , "Ptr", memDC
        , "Int", 0, "Int", 0
        , "Int", magnifierSize, "Int", magnifierSize
        , "Ptr", memDC
        , "Int", 0, "Int", 0
        , "UInt", 0x00FFFFFF)
    
    sourceSize := magnifierSize // magnificationFactor
    
    ; ensure sourceSize is odd for perfect centering
    if (Mod(sourceSize, 2) == 0)
        sourceSize += 1
        
    ; make sure the cursor is exactly in the center of the captured area
    sourceX := x - (sourceSize // 2)
    sourceY := y - (sourceSize // 2)
    
    xOffset := 0
    yOffset := 0
    
    if (sourceX < 0) {
        xOffset := -sourceX
        sourceX := 0
    }
    if (sourceY < 0) {
        yOffset := -sourceY
        sourceY := 0
    }
    if (sourceX + sourceSize > screenWidth) {
        xOffset := screenWidth - (sourceX + sourceSize)
        sourceX := screenWidth - sourceSize
    }
    if (sourceY + sourceSize > screenHeight) {
        yOffset := screenHeight - (sourceY + sourceSize)
        sourceY := screenHeight - sourceSize
    }
    
    destWidth := magnifierSize - (borderWidth * 2)
    destHeight := magnifierSize - (borderWidth * 2) - textHeight
    
    DllCall("gdi32\StretchBlt"
        , "Ptr", memDC
        , "Int", borderWidth, "Int", borderWidth
        , "Int", destWidth, "Int", destHeight
        , "Ptr", screenDC
        , "Int", sourceX, "Int", sourceY
        , "Int", sourceSize, "Int", sourceSize
        , "UInt", 0x00CC0020)
    
    ; calculate the exact center of the magnified area
    ; when at screen edges adjust the position to match the actual cursor position
    scaledX := destWidth / 2 + borderWidth
    scaledY := destHeight / 2 + borderWidth
    
    if (xOffset != 0 || yOffset != 0) {
        relativeX := x - sourceX
        relativeY := y - sourceY
        
        scaledX := (relativeX * destWidth) / sourceSize + borderWidth
        scaledY := (relativeY * destHeight) / sourceSize + borderWidth
    }
    
    scaledX := Round(scaledX)
    scaledY := Round(scaledY)
    
    ; white border for dot
    whiteBorderSize := 4
    whitePen := DllCall("CreatePen", "Int", 0, "Int", 1, "UInt", 0xFFFFFF, "Ptr")
    whiteBrush := DllCall("CreateSolidBrush", "UInt", 0xFFFFFF, "Ptr")
    oldPen := DllCall("SelectObject", "Ptr", memDC, "Ptr", whitePen)
    oldBrush := DllCall("SelectObject", "Ptr", memDC, "Ptr", whiteBrush)
    
    DllCall("Ellipse", "Ptr", memDC, 
            "Int", scaledX - whiteBorderSize, "Int", scaledY - whiteBorderSize, 
            "Int", scaledX + whiteBorderSize, "Int", scaledY + whiteBorderSize)
    
    DllCall("SelectObject", "Ptr", memDC, "Ptr", oldPen)
    DllCall("SelectObject", "Ptr", memDC, "Ptr", oldBrush)
    DllCall("DeleteObject", "Ptr", whitePen)
    DllCall("DeleteObject", "Ptr", whiteBrush)
    
    ; black dot
    centerSize := 3
    blackPen := DllCall("CreatePen", "Int", 0, "Int", 1, "UInt", 0x000000, "Ptr")
    centerBrush := DllCall("CreateSolidBrush", "UInt", 0x000000, "Ptr") ; Black center
    oldPen := DllCall("SelectObject", "Ptr", memDC, "Ptr", blackPen)
    oldBrush := DllCall("SelectObject", "Ptr", memDC, "Ptr", centerBrush)
    
    DllCall("Ellipse", "Ptr", memDC, 
            "Int", scaledX - centerSize, "Int", scaledY - centerSize, 
            "Int", scaledX + centerSize, "Int", scaledY + centerSize)
    
    DllCall("SelectObject", "Ptr", memDC, "Ptr", oldPen)
    DllCall("SelectObject", "Ptr", memDC, "Ptr", oldBrush)
    DllCall("DeleteObject", "Ptr", blackPen)
    DllCall("DeleteObject", "Ptr", centerBrush)
    
    ; white border crosshair
    whitePen := DllCall("CreatePen", "Int", 0, "Int", 5, "UInt", 0xFFFFFF, "Ptr")
    oldPen := DllCall("SelectObject", "Ptr", memDC, "Ptr", whitePen)

    ; horizontal white line
    DllCall("gdi32\MoveToEx", "Ptr", memDC, "Int", borderWidth, "Int", scaledY, "Ptr", 0)
    DllCall("gdi32\LineTo", "Ptr", memDC, "Int", scaledX - 8, "Int", scaledY)

    DllCall("gdi32\MoveToEx", "Ptr", memDC, "Int", scaledX + 8, "Int", scaledY, "Ptr", 0)
    DllCall("gdi32\LineTo", "Ptr", memDC, "Int", destWidth, "Int", scaledY)

    ; vertical white line
    DllCall("gdi32\MoveToEx", "Ptr", memDC, "Int", scaledX, "Int", borderWidth, "Ptr", 0)
    DllCall("gdi32\LineTo", "Ptr", memDC, "Int", scaledX, "Int", scaledY - 8)

    DllCall("gdi32\MoveToEx", "Ptr", memDC, "Int", scaledX, "Int", scaledY + 8, "Ptr", 0)
    DllCall("gdi32\LineTo", "Ptr", memDC, "Int", scaledX, "Int", destHeight)

    ; clean up white pen
    DllCall("SelectObject", "Ptr", memDC, "Ptr", oldPen)
    DllCall("DeleteObject", "Ptr", whitePen)

    ; black crosshair
    blackPen := DllCall("CreatePen", "Int", 0, "Int", 3, "UInt", 0x000000, "Ptr")
    oldPen := DllCall("SelectObject", "Ptr", memDC, "Ptr", blackPen)

    ; horizontal black line
    DllCall("gdi32\MoveToEx", "Ptr", memDC, "Int", borderWidth + 1, "Int", scaledY, "Ptr", 0)
    DllCall("gdi32\LineTo", "Ptr", memDC, "Int", scaledX - 8, "Int", scaledY)

    DllCall("gdi32\MoveToEx", "Ptr", memDC, "Int", scaledX + 8, "Int", scaledY, "Ptr", 0)
    DllCall("gdi32\LineTo", "Ptr", memDC, "Int", destWidth - 1, "Int", scaledY)

    ; vertical black line
    DllCall("gdi32\MoveToEx", "Ptr", memDC, "Int", scaledX, "Int", borderWidth + 1, "Ptr", 0)
    DllCall("gdi32\LineTo", "Ptr", memDC, "Int", scaledX, "Int", scaledY - 8)

    DllCall("gdi32\MoveToEx", "Ptr", memDC, "Int", scaledX, "Int", scaledY + 8, "Ptr", 0)
    DllCall("gdi32\LineTo", "Ptr", memDC, "Int", scaledX, "Int", destHeight - 1)

    ; clean up black pen
    DllCall("SelectObject", "Ptr", memDC, "Ptr", oldPen)
    DllCall("DeleteObject", "Ptr", blackPen)
    
    brush := DllCall("CreateSolidBrush", "UInt", 0xFFFFFF, "Ptr")
    oldBrush := DllCall("SelectObject", "Ptr", memDC, "Ptr", brush)
    DllCall("Rectangle", "Ptr", memDC, "Int", 0, "Int", magnifierSize - textHeight, "Int", magnifierSize, "Int", magnifierSize)
    DllCall("SelectObject", "Ptr", memDC, "Ptr", oldBrush)
    DllCall("DeleteObject", "Ptr", brush)
    
    DllCall("ReleaseDC", "Ptr", 0, "Ptr", screenDC)
    
    DllCall("BitBlt", "Ptr", magnifierDC, "Int", 0, "Int", 0, "Int", magnifierSize, "Int", magnifierSize, "Ptr", memDC, "Int", 0, "Int", 0, "UInt", 0x00CC0020)
    
    colorText.Value := colorValue
    
    textColor := GetContrastingColor(currentColor)
    colorText.SetFont("c" . textColor)
    
    colorText.Opt("Background" . SubStr(currentColor, 3))
    
    magnifierGui.Show("NA x" . magnifierX . " y" . magnifierY)
}

; Get contrasting color (black or white) for better text visibility
GetContrastingColor(color) {
    if (SubStr(color, 1, 2) = "0x")
        color := SubStr(color, 3)
    
    r := Integer("0x" . SubStr(color, 1, 2))
    g := Integer("0x" . SubStr(color, 3, 2))
    b := Integer("0x" . SubStr(color, 5, 2))
    
    brightness := (r * 299 + g * 587 + b * 114) / 1000
    
    return (brightness > 128) ? "000000" : "FFFFFF"
}

; Format color based on current index
FormatColor(color, formatIndex) {
    if (SubStr(color, 1, 2) = "0x")
        color := SubStr(color, 3)
    
    r := Integer("0x" . SubStr(color, 1, 2))
    g := Integer("0x" . SubStr(color, 3, 2))
    b := Integer("0x" . SubStr(color, 5, 2))
    
    switch formatIndex {
        case 1:
            return "#" . color
        case 2:
            return "rgb(" . r . "," . g . "," . b . ")"
        case 3:
            return "rgba(" . r . "," . g . "," . b . ",1)"
        case 4:
            hsl := RgbToHsl(r, g, b)
            return "hsl(" . Round(hsl.h) . "," . Round(hsl.s) . "%," . Round(hsl.l) . "%)"
        case 5:
            hsl := RgbToHsl(r, g, b)
            return "hsla(" . Round(hsl.h) . "," . Round(hsl.s) . "%," . Round(hsl.l) . "%,1)"
    }
}

; Convert RGB to HSL
RgbToHsl(r, g, b) {
    r /= 255
    g /= 255
    b /= 255
    
    maxVal := r
    if (g > maxVal)
        maxVal := g
    if (b > maxVal)
        maxVal := b
        
    minVal := r
    if (g < minVal)
        minVal := g
    if (b < minVal)
        minVal := b
    
    l := (maxVal + minVal) / 2
    
    if (maxVal = minVal) {
        h := 0
        s := 0
    } else {
        s := (l > 0.5) ? (maxVal - minVal) / (2 - maxVal - minVal) : (maxVal - minVal) / (maxVal + minVal)
        
        if (maxVal = r)
            h := (g - b) / (maxVal - minVal) + (g < b ? 6 : 0)
        else if (maxVal = g)
            h := (b - r) / (maxVal - minVal) + 2
        else
            h := (r - g) / (maxVal - minVal) + 4
        
        h *= 60
    }
    
    return { h: h, s: s * 100, l: l * 100 }
}

CatchColor(useAltFormat := false) {
    global colorFormatIndex
    
    CoordMode("Mouse", "Screen")
    CoordMode("Pixel", "Screen")
    MouseGetPos(&x, &y)
    
    color := PixelGetColor(x, y, "RGB")
    color := "0x" . SubStr(color, 3)
    
    if (useAltFormat)
        colorFormatIndex := Mod(colorFormatIndex, 5) + 1
    
    outMsg := FormatColor(color, colorFormatIndex)
    
    A_Clipboard := outMsg
    
    ; Hide the magnifier and restore cursors immediately
    magnifierGui.Hide()
    RestoreCursors()
    ToolTip("")
    
    ; Show notification and wait for it to complete
    ShowNotificationAndExit(outMsg, color, colorFormatIndex)
}

; Show notification and exit when done
ShowNotificationAndExit(colorValue, hexColor, formatIndex) {
    ShowCustomNotification(colorValue, hexColor, formatIndex)
    ExitApp()
}

; Show a custom notification with the picked color
ShowCustomNotification(colorValue, hexColor, formatIndex) {
    notifyGui := Gui("-Caption +AlwaysOnTop +ToolWindow +E0x20")
    notifyGui.SetFont("s12", "Segoe UI")
    notifyGui.BackColor := "1E1E1E"
    
    width := Max(300, StrLen(colorValue) * 12)
    
    height := 110
    
    r := Integer("0x" . SubStr(hexColor, 3, 2))
    g := Integer("0x" . SubStr(hexColor, 5, 2))
    b := Integer("0x" . SubStr(hexColor, 7, 2))
    
    colorBorder := notifyGui.Add("Progress", "x0 y0 w6 h" . height . " Background" . SubStr(hexColor, 3))
    
    colorBox := notifyGui.Add("Progress", "x20 y25 w40 h40 Background" . SubStr(hexColor, 3))
    
    titleText := notifyGui.Add("Text", "x70 y15 w" . (width - 90) . " h24 c00BFFF", "XPicker")
    titleText.SetFont("s14 bold")
    
    valueText := notifyGui.Add("Text", "x70 y45 w" . (width - 90) . " h30 cFFFFFF", GetShortColorValue(colorValue))
    valueText.SetFont("s11")
    
    screenWidth := A_ScreenWidth
    screenHeight := A_ScreenHeight
    xPos := screenWidth - width - 20
    yPos := screenHeight - height - 40
    
    notifyGui.Show("w" . width . " h" . height . " x" . xPos . " y" . yPos . " NoActivate")
    ApplyRoundedCorners(notifyGui.Hwnd, 10)
    
    ; Starting with fade in
    WinSetTransparent(0, "ahk_id " . notifyGui.Hwnd)
    FadeIn(notifyGui.Hwnd)
    
    ; Wait and then fade out
    Sleep(2000)
    FadeOut(notifyGui.Hwnd)
    notifyGui.Destroy()
}

; Fade in animation
FadeIn(hwnd) {
    Loop 10 {
        opacity := A_Index * 25
        WinSetTransparent(Integer(opacity), "ahk_id " . hwnd)
        Sleep(15)
    }
}

; Fade out animation
FadeOut(hwnd) {
    Loop 10 {
        opacity := 250 - (A_Index * 25)
        WinSetTransparent(Integer(opacity), "ahk_id " . hwnd)
        Sleep(30)
    }
}

ApplyRoundedCorners(hwnd, radius) {
    region := DllCall("CreateRoundRectRgn", "Int", 0, "Int", 0, "Int", 300, "Int", 90, "Int", radius, "Int", radius, "Ptr")
    
    DllCall("SetWindowRgn", "Ptr", hwnd, "Ptr", region, "Int", True)
    
    DllCall("DeleteObject", "Ptr", region)
}

GetShortColorValue(colorValue) {
    if (StrLen(colorValue) > 40)
        return SubStr(colorValue, 1, 37) . "..."
    return colorValue
}

HexToRGB(color, mode := "") {
    r := SubStr(color, 3, 2)
    g := SubStr(color, 5, 2)
    b := SubStr(color, 7, 2)
    
    if (mode = "Message")
        return "R: " . r . ", G: " . g . ", B: " . b
    else if (mode = "RGB")
        return r . "," . g . "," . b
    else
        return "#" . SubStr(color, 3)
}

RestoreCursors(*) {
    DllCall("SystemParametersInfo", "UInt", 0x57, "UInt", 0, "Ptr", 0, "UInt", 0)
}

; Set the system cursor
SetSystemCursor(cursor) {
    static systemCursors := Map(
        "IDC_APPSTARTING", 32650,
        "IDC_ARROW", 32512,
        "IDC_CROSS", 32515,
        "IDC_HAND", 32649,
        "IDC_HELP", 32651,
        "IDC_IBEAM", 32513,
        "IDC_NO", 32648,
        "IDC_SIZEALL", 32646,
        "IDC_SIZENESW", 32643,
        "IDC_SIZENS", 32645,
        "IDC_SIZENWSE", 32642,
        "IDC_SIZEWE", 32644,
        "IDC_UPARROW", 32516,
        "IDC_WAIT", 32514
    )
    
    if !systemCursors.Has(cursor) {
        MsgBox("Invalid cursor name: " . cursor)
        return
    }
    
    cursorID := systemCursors[cursor]
    cursorHandle := DllCall("LoadCursor", "UInt", 0, "Int", cursorID, "Ptr")
    
    for curName, sysID in systemCursors {
        copyCursor := DllCall("CopyImage", "Ptr", cursorHandle, "UInt", 2, "Int", 0, "Int", 0, "UInt", 0, "Ptr")
        DllCall("SetSystemCursor", "Ptr", copyCursor, "Int", sysID)
    }
} 
