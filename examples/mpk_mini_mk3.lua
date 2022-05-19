-- Example script for the Akai MPK Mini MK3
-- Why this controller? it's the one I have

-- This maps the analog stick to an Xbox gamepad
-- requires that up and down be bound to pitch bend and left and right be bound to CC1 and CC2

local pad = nil

function on_script_init()
    midi.open(1)
    pad = gamepad.create()
end

local js = {x=0, y=0}

function on_midi_recv(evt)
    if evt.event == "control_change" then
        if evt.control == 1 then
            js.x = -(evt.value / 127.0)
        elseif evt.control == 2 then
            js.x = (evt.value / 127.0)
        end
    end
    if evt.event == "pitch_bend" then
        local v = (evt.value - 16384) * -1
        js.y = v / 16384.0
    end

    if evt.is_note then
        if evt.key == 40 then
            pad.button(gamepad.BTN_A, evt.event == "note_on")
        end
    end

    pad.axis(gamepad.AXIS_LSTICK_X, js.x)
    pad.axis(gamepad.AXIS_LSTICK_Y, js.y)
end