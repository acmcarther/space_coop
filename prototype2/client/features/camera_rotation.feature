Feature: Rotating the camera
  Background:
    Given an engine with:
      | camera::PreprocessorSystem |
      | camera::MovementSystem     |
    Then the camera pos is the default pos
    And the camera target distance invariant holds

  Scenario: The camera can move left
    When the mouse is moved left
    And the engine runs once
    Then the camera moves left
    And the camera target distance invariant holds

  Scenario: The camera can move right
    When the mouse is moved right
    And the engine runs once
    Then the camera moves right
    And the camera target distance invariant holds
