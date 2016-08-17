Feature: WASD player controls
  Background:
    Given an engine with:
      | pause::System              |
      | camera::MovementSystem     |
      | player::MoveSystem         |
      | player::PreprocessorSystem |
    And we monitor outgoing network events
    And the camera is at 1, 0, 0

  Scenario: W emits the forward moving event
    When the 'W' key is pressed
    And the engine runs once
    Then the following network events are emitted:
      | DomainEvent | SelfMove | -0.1 | 0 | 0 |

  Scenario: A emits the left moving event
    When the 'A' key is pressed
    And the engine runs once
    Then the following network events are emitted:
      | DomainEvent | SelfMove | 0 | -0.1 | 0 |

  Scenario: S emits the backward moving event
    When the 'S' key is pressed
    And the engine runs once
    Then the following network events are emitted:
      | DomainEvent | SelfMove | 0.1 | 0 | 0 |

  Scenario: D emits the right moving event
    When the 'D' key is pressed
    And the engine runs once
    Then the following network events are emitted:
      | DomainEvent | SelfMove | 0 | 0.1 | 0 |

  Scenario: The movements are camera relative
    Given the camera is at 2, 2, 0
    When the 'W' key is pressed
    And the engine runs once
    Then the following network events are emitted:
      | DomainEvent | SelfMove | -0.07071068 | -0.07071068 | 0 |

  Scenario: More than one may be emitted at once
    When the 'W' key is pressed
    And the 'S' key is pressed
    And the engine runs once
    Then the following network events are emitted:
      | DomainEvent | SelfMove | 0.1 | 0 | 0 |
      | DomainEvent | SelfMove | -0.1 | 0 | 0 |

  Scenario: No movement keys work while paused
    When the game is set as paused
    And the 'W' key is pressed
    And the 'A' key is pressed
    And the 'S' key is pressed
    And the 'D' key is pressed
    And the engine runs once
    Then no network events are emitted
