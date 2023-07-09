function isUndefined(v) { return typeof v === "undefined"; }
function isFunction(v) { return typeof v === "function"; }
function isBoolean(v) { return typeof v === "boolean"; }
function isArray(v) { return Array.isArray(v); }
function isFiniteNumeric(v) { return Number.isFinite(v); }
function isValidDate(v) { return v instanceof Date && !isNaN(v); }

function isString(v) { return typeof v === "string" /* v = "s" */ || v instanceof String /* v = new String("s") */; }
function isNonBlankString(v) { return isString(v) && v.trim().length !== 0; }
function isBlankString(v) { return isString(v) && v.trim().length === 0; }
function isStringFiniteNumeric(v) {
  if (!isString(v)) return false;

  return isFinite(v) && // Check _entirety_ string using coercion (e.g., "" becomes 0 or true/false becomes 1/0, see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/isNaN#description)
          Number.isFinite(parseFloat(v)); // Reject coercion cases using parseFloat. parseFloat returns NaN for empty string and true/false
}

/// Trim input string. Return empty string when input is not a string.
function trimAsString(v) { return isString(v) ? v.trim() : ""; }

function setupKeydownEvent(tx) {
    if (!tx) {
        console.error("Missing tx, keydown event won't be handled");
        return;
    }

    document.addEventListener("keydown", function(event) {
        // From: https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent
        const rustEvent = {
            "altKey": event.altKey,
            "code": event.code,
            "ctrlKey": event.ctrlKey,
            "isComposing": event.isComposing,
            "key": event.key,
            "location": event.location,
            "metaKey": event.metaKey,
            "repeat": event.repeat,
            "shiftKey": event.shiftKey,
        };
        tx.send(rustEvent);
    });
}

function setTheme(lightMode) {
    console.log("Change theme: ", lightMode)
    if (lightMode) {
        document.documentElement.setAttribute('data-bs-theme', 'light');
    } else {
        document.documentElement.setAttribute('data-bs-theme', 'dark');
    }
}

$(document).ready(function() {

    // Resize flex elements.
    // From: https://stackoverflow.com/questions/28767221/flexbox-resizing
    function createResizeHandler(resizerElementCheckFn) {
        if (!isFunction(resizerElementCheckFn)) {
            resizerElementCheckFn = (e) => true;
        }

        function beginResizing(mdEvent) {
            const resizerElem = mdEvent.target;
            if (!resizerElementCheckFn(resizerElem)) {
                return;
            }

            const parentElem = resizerElem.parentNode;
            const parentStyle = getComputedStyle(parentElem);
            if (parentStyle.display !== 'flex') {
                return;
            }

            function getInfoFromDirection(direction) {
                switch (direction) {
                case 'row':
                    return [resizerElem.previousElementSibling, resizerElem.nextElementSibling, 'offsetWidth',  'pageX'];
                case 'column':
                    return [resizerElem.previousElementSibling, resizerElem.nextElementSibling, 'offsetHeight', 'pageY']
                case 'row-reverse':
                    return [resizerElem.nextElementSibling, resizerElem.previousElementSibling, 'offsetWidth',  'pageX']
                case 'column-reverse':
                    return [resizerElem.nextElementSibling, resizerElem.previousElementSibling, 'offsetHeight', 'pageY']
                default:
                    console.error(`internal error: unimplemented resizer for ${direction}`);
                    return undefined;
                }
            }
            const direction = parentStyle['flex-direction'];
            const info = getInfoFromDirection(direction);
            if (info === undefined) return;

            const [prevElem, nextElem, sizeProp, posProp] = info;
            if (!prevElem || !nextElem) return;

            mdEvent.preventDefault();

            // Avoid cursor flickering (reset in onMouseUp)
            document.body.style.cursor = getComputedStyle(resizerElem).cursor;
        
            let prevSize = prevElem[sizeProp];
            let nextSize = nextElem[sizeProp];
            const sumSize = prevSize + nextSize;
            const prevGrow = Number(prevElem.style.flexGrow);
            const nextGrow = Number(nextElem.style.flexGrow);
            const sumGrow = prevGrow + nextGrow;
            let lastPos = mdEvent[posProp];
        
            function onMouseMove(mmEvent) {
                mmEvent.preventDefault();

                let pos = mmEvent[posProp];
                const distance = pos - lastPos; // This can be negative
                let prevNewSize = prevSize + distance;
                let nextNewSize = nextSize - distance;
                if (prevNewSize < 0) {
                    nextNewSize = sumSize;
                    pos -= prevNewSize;
                    prevNewSize = 0;
                }
                if (nextNewSize < 0) {
                    prevNewSize = sumSize;
                    pos += nextNewSize;
                    nextNewSize = 0;
                }
                prevSize = Math.min(prevNewSize, sumSize);
                nextSize = Math.min(nextNewSize, sumSize);

                const prevGrowNew = sumGrow * (prevSize / sumSize);
                const nextGrowNew = sumGrow * (nextSize / sumSize);
        
                prevElem.style.flexGrow = prevGrowNew;
                nextElem.style.flexGrow = nextGrowNew;

                lastPos = pos;
            }
        
            function onMouseUp(muEvent) {
                // Reset cursor state that was used to avoid cursor flickering
                document.body.style.removeProperty('cursor');
                
                window.removeEventListener('mousemove', onMouseMove);
                window.removeEventListener('mouseup', onMouseUp);
            }
        
            window.addEventListener('mousemove', onMouseMove);
            window.addEventListener('mouseup', onMouseUp);
        }

        return beginResizing;
    }
    $(document).on('mousedown', '.lv-resizer', createResizeHandler((elm) => elm.classList.contains('lv-resizer')));

});
