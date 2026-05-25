import 'package:json_annotation/json_annotation.dart';

part 'srp_proposal.g.dart';

@JsonSerializable()
class SrpProposal {
  final String slug;
  final String name;
  final String description;
  @JsonKey(name: 'schema_json')
  final Map<String, dynamic>? schemaJson;
  @JsonKey(name: 'how_it_works')
  final String howItWorks;
  @JsonKey(name: 'compiled_code')
  final String compiledCode;
  final String status;
  final List<String> intents;
  @JsonKey(name: 'error_logs')
  final String? errorLogs;

  SrpProposal({
    required this.slug,
    required this.name,
    required this.description,
    this.schemaJson,
    required this.howItWorks,
    required this.compiledCode,
    required this.status,
    required this.intents,
    this.errorLogs,
  });

  factory SrpProposal.fromJson(Map<String, dynamic> json) => _$SrpProposalFromJson(json);
  Map<String, dynamic> toJson() => _$SrpProposalToJson(this);
}
