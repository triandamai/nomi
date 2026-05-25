import 'package:json_annotation/json_annotation.dart';

part 'reinforcement_state.g.dart';

@JsonSerializable()
class ReinforcementState {
  final String slug;
  @JsonKey(name: 'enriched_description')
  final String enrichedDescription;
  @JsonKey(name: 'additional_rules')
  final List<String> additionalRules;
  @JsonKey(name: 'learned_phrases')
  final List<String> learnedPhrases;

  ReinforcementState({
    required this.slug,
    required this.enrichedDescription,
    required this.additionalRules,
    required this.learnedPhrases,
  });

  factory ReinforcementState.fromJson(Map<String, dynamic> json) => _$ReinforcementStateFromJson(json);
  Map<String, dynamic> toJson() => _$ReinforcementStateToJson(this);
}
