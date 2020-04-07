 # Muses Material Survey

Currently the following survey page types are supported:

   * Front matter
   * Consent
   * Likert 
   * Press, aimed at capturing a material press for pressure
   * Slider, aimed at capturing movements across the material
   * Reponses

# Config

There is a single config.json file, which contains the following fields:

```javascript
  {
     "id": "number",
     "likert_dir": "string",
     "gesture_dir": "string",
     "responses_dir": "string",
  }
```
   * id - is the next unique ID given to each participant
   * likert_dir - directory for Likert CSV files
   * gesture_dir - directory for Gesture CSV files
   * responses_dir - directory for Responses CSV files

# CSV Representation of Results

All data is stored in CSV files. Corresponding to each of the path enteries in the previous section,
we then have for each survey:

	* likert/ID.csv           - Likert results for the survey with ID
	* gestures/ID.csv.    - Gesture results for the survey with ID (including all touch information)
	* responses/ID.csv  -  Response results for the survey with ID

# TODO:

   * Tap
   * Gesture CSV
   * Response CSV
   * Visualization of Likert with R

# PROTOCOL

## Client to server messages

#### Connection made

The following message is sent just once by the web-client on creation of the
websocket connection between front and backend.

```javascript
{ "type": "connection" }
```

#### Begin survey 

The following message is sent just once, when the user presses begin button on
frontpage.

```javascript
{ "type": "begin" }
```

#### Consent

The following message is sent when the user presses the consent button.

```javascript
{ "type": "consent" }
```

#### Likert

```javascript
{ "type": "likert", "name": "string", "value": "number" }
```

## Server to client

#### Consent

```javascript
{ "type": "consentID", "id": "number"}
```

#### Press

Set the radius of circle and ring of press animation.

```javascript
{ "type": "press", "circle": "number", "ring": "number"}
```

#### Slider

Set user and box position of slider animation.

```javascript
{ "type": "slider", "user_x": "number", "user_y": "number", "box_x": "number", "box_y": "number"}
```

#### Material Type

Set the current material type of Likert.

```javascript
{ "type": "materialIndex", "value": "number", "slide": "number"}
```

Value is the inded of the selected material.

Slide is the slide number material should be set for. (See goto slide for details of valid numbers.)

#### Gesture Type

Set the current gesture type, e.g. "TAP".

```javascript
{ "type": "gestureType", "value": "string"}
```

#### Goto slide

View moves to a given slide.

```javascript
{ "type": "goto", "slide": "number" }
```

Slide numbers are defined as follows:

   * 0 - Front mattter
   * 1 - Consent
   * 2 - Likert
   * 3 - Press
   * 4 - Material you felt was the most accurate
   * 5 - Please touch the material you felt was the most comfortable to use
   * 6 - Please touch the material you felt was the most responsive
   * 7 - Please order the materials according to your preferences
   * 8 - Slider
   * X - Closing matter

# CSV Format

 id=id, press, material, circleRadius, ringRadius, (timestamp, pressure, x, y), ...