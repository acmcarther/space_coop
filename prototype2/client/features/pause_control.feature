Feature: Pausing and Unpausing
  Background:
    Given an engine with:
      | pause::System |

  Scenario: Hitting esc while paused
    Given the game is set as paused
    When the 'ESC' key is pressed
    And the engine runs once
    Then the game is unpaused

  Scenario: Hitting esc while unpaused
    Given the game is set as unpaused
    When the 'ESC' key is pressed
    And the engine runs once
    Then the game is paused
